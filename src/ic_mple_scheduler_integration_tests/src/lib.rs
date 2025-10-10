use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use candid::CandidType;
use ic_cdk::{init, post_upgrade, query, update};
use ic_mple_scheduler::SchedulerError;
use ic_mple_scheduler::scheduler::{Scheduler, TaskScheduler};
use ic_mple_scheduler::task::{InnerScheduledTask, ScheduledTask, Task, TaskStatus};
use ic_mple_structures::DefaultMemoryImpl;
use ic_mple_structures::{MemoryId, MemoryManager, StableBTreeMap, StableCell, VirtualMemory};
use serde::{Deserialize, Serialize};

type Storage = StableBTreeMap<u64, InnerScheduledTask<DummyTask>, VirtualMemory<DefaultMemoryImpl>>;
type Sequence = StableCell<u64, VirtualMemory<DefaultMemoryImpl>>;
type PanickingScheduler = Scheduler<DummyTask, Storage, Sequence>;

const SCHEDULER_STORAGE_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    pub static MEMORY_MANAGER: MemoryManager<DefaultMemoryImpl> = MemoryManager::init(DefaultMemoryImpl::default());

    static SCHEDULER: RefCell<PanickingScheduler> = {
        let map: Storage = Storage::new(MEMORY_MANAGER.with(|mm| mm.get(SCHEDULER_STORAGE_MEMORY_ID)));
        let sequence: Sequence = Sequence::new(MEMORY_MANAGER.with(|mm| mm.get(SCHEDULER_STORAGE_MEMORY_ID)), 0);

        let mut scheduler = PanickingScheduler::new(
            map,
            sequence,
        );

        scheduler.set_running_task_timeout(30);
        scheduler.on_completion_callback(save_state_cb);

        RefCell::new(scheduler)
    };

    static COMPLETED_TASKS: RefCell<Vec<u64>> = const { RefCell::new(vec![]) };
    static FAILED_TASKS: RefCell<Vec<u64>> = const { RefCell::new(vec![]) };
    static PANICKED_TASKS : RefCell<Vec<u64>> = const { RefCell::new(vec![]) };

}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum DummyTask {
    Panicking,
    GoodTask,
    FailTask,
}

impl Task for DummyTask {
    type Ctx = ();

    fn execute(
        &self,
        _: Self::Ctx,
        _task_scheduler: Box<dyn 'static + TaskScheduler<Self>>,
    ) -> Pin<Box<dyn Future<Output = Result<(), SchedulerError>>>> {
        match self {
            Self::GoodTask => Box::pin(async move { Ok(()) }),
            Self::Panicking => Box::pin(async move {
                panic!("PanicTask::execute");
            }),
            Self::FailTask => Box::pin(async move {
                Err(SchedulerError::TaskExecutionFailed(
                    "i dunno why".to_string(),
                ))
            }),
        }
    }
}

#[init]
pub fn init() {
    set_timers();
}

#[post_upgrade]
pub fn post_upgrade() {
    set_timers();
}

fn set_timers() {
    ic_cdk_timers::set_timer_interval(Duration::from_millis(10), || do_run_scheduler());
}

#[query]
pub fn panicked_tasks() -> Vec<u64> {
    PANICKED_TASKS.with_borrow(|tasks| tasks.clone())
}

#[query]
pub fn completed_tasks() -> Vec<u64> {
    COMPLETED_TASKS.with_borrow(|tasks| tasks.clone())
}

#[query]
pub fn failed_tasks() -> Vec<u64> {
    FAILED_TASKS.with_borrow(|tasks| tasks.clone())
}

#[query]
pub fn get_task(task_id: u64) -> Option<InnerScheduledTask<DummyTask>> {
    let scheduler = SCHEDULER.with_borrow(|scheduler| scheduler.clone());
    scheduler.get_task(task_id)
}

#[update]
pub fn schedule_tasks(tasks: Vec<DummyTask>) -> Vec<u64> {
    let scheduler = SCHEDULER.with_borrow(|scheduler| scheduler.clone());
    let scheduled_tasks = tasks.into_iter().map(ScheduledTask::new).collect();
    scheduler.append_tasks(scheduled_tasks)
}

#[update]
pub fn run_scheduler() {
    do_run_scheduler();
}

fn do_run_scheduler() {
    ic_cdk::println!("run_scheduler");
    let scheduler = SCHEDULER.with_borrow(|scheduler| scheduler.clone());
    scheduler.run(()).unwrap();
}

fn save_state_cb(task: InnerScheduledTask<DummyTask>) {
    match task.status() {
        TaskStatus::Waiting { .. } => {}
        TaskStatus::Completed { .. } => {
            COMPLETED_TASKS.with_borrow_mut(|tasks| {
                tasks.push(task.id());
            });
        }
        TaskStatus::Running { .. } => {}
        TaskStatus::Failed { .. } => {
            FAILED_TASKS.with_borrow_mut(|tasks| {
                tasks.push(task.id());
            });
        }
        TaskStatus::TimeoutOrPanic { .. } => {
            PANICKED_TASKS.with_borrow_mut(|tasks| {
                tasks.push(task.id());
            });
        }
        TaskStatus::Scheduled { .. } => {}
    };
}
