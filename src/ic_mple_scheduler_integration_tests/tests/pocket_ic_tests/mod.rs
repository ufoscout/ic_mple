mod scheduler;
mod wasm_utils;

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use candid::{CandidType, Encode, Principal};
use ic_mple_client::{CanisterClient, PocketIcClient};
use ic_mple_pocket_ic::get_pocket_ic_client;
use ic_mple_pocket_ic::pocket_ic::nonblocking::PocketIc;
use ic_mple_scheduler::scheduler::TaskScheduler;
use ic_mple_scheduler::task::{InnerScheduledTask, Task};
use ic_mple_scheduler::SchedulerError;
use serde::{Deserialize, Serialize};
use wasm_utils::get_dummy_scheduler_canister_bytecode;

pub fn alice() -> Principal {
    Principal::from_text("sgymv-uiaaa-aaaaa-aaaia-cai").unwrap()
}

pub struct PocketIcTestContext {
    canister_client: PocketIcClient,
    pub dummy_scheduler_canister: Principal,
}

impl PocketIcTestContext {
    /// Returns the PocketIC client for the canister.
    pub fn client(&self) -> &PocketIc {
        self.canister_client.client()
    }

    pub async fn get_task(&self, task_id: u64) -> Option<InnerScheduledTask<DummyTask>> {
        self.canister_client
            .query("get_task", (task_id,))
            .await
            .unwrap()
    }

    pub async fn completed_tasks(&self) -> Vec<u64> {
        self.canister_client
            .query("completed_tasks", ())
            .await
            .unwrap()
    }

    pub async fn panicked_tasks(&self) -> Vec<u64> {
        self.canister_client
            .query("panicked_tasks", ())
            .await
            .unwrap()
    }

    pub async fn failed_tasks(&self) -> Vec<u64> {
        self.canister_client
            .query("failed_tasks", ())
            .await
            .unwrap()
    }

    pub async fn schedule_tasks(&self, tasks: Vec<DummyTask>) -> Vec<u64> {
        self.canister_client
            .update("schedule_tasks", (tasks,))
            .await
            .unwrap()
    }

    pub async fn run_scheduler(&self) {
        self.client()
            .advance_time(Duration::from_millis(5000))
            .await;
        self.client().tick().await;
    }
}

async fn deploy_dummy_scheduler_canister() -> PocketIcTestContext {
    let client = get_pocket_ic_client().await.build_async().await;
    println!("Creating dummy canister");

    let sender = alice();
    let canister = client
        .create_canister_with_settings(Some(sender), None)
        .await;
    println!("Canister created with principal {}", canister);

    let canister_client =
        PocketIcClient::from_client(client, canister, alice());

    let env = PocketIcTestContext {
        canister_client,
        dummy_scheduler_canister: canister,
    };

    env.client().add_cycles(canister, 10_u128.pow(14)).await;
    println!("cycles added");

    let dummy_wasm = get_dummy_scheduler_canister_bytecode();
    let args = Encode!(&()).unwrap();
    env.client()
        .install_canister(canister, dummy_wasm.to_vec(), args, Some(sender))
        .await;

    println!("Installed dummy canister");

    env
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DummyTask {
    Panicking,
    GoodTask,
    FailTask,
}

impl Task for DummyTask {
    type Ctx = ();

    fn execute(
        &self,
        _: (),
        _task_scheduler: Box<dyn 'static + TaskScheduler<Self>>,
    ) -> Pin<Box<dyn Future<Output = Result<(), SchedulerError>>>> {
        Box::pin(async move { Ok(()) })
    }
}
