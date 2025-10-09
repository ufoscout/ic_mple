use ic_cdk::{init, query, update};
use service::Service;

use crate::did::{BoundedTransaction, UnboundedTransaction};

mod service;

    #[init]
    pub fn init() {
        Service::init()
    }

    #[query]
    pub fn get_tx_from_btreemap(key: u64) -> Option<BoundedTransaction> {
        Service::get_tx_from_btreemap(key)
    }

    #[update]
    pub async fn insert_tx_to_btreemap(transaction: BoundedTransaction) -> u64 {
        Service::insert_tx_to_btreemap(transaction)
    }

    #[query]
    pub fn get_tx_from_cached_btreemap(key: u64) -> Option<BoundedTransaction> {
        Service::get_tx_from_cached_btreemap(key)
    }

    #[update]
    pub async fn insert_tx_to_cached_btreemap(transaction: BoundedTransaction) -> u64 {
        Service::insert_tx_to_cached_btreemap(transaction)
    }

    #[query]
    pub fn get_tx_from_cell() -> BoundedTransaction {
        Service::get_tx_from_cell()
    }

    #[update]
    pub async fn insert_tx_to_cell(transaction: BoundedTransaction) -> BoundedTransaction {
        Service::insert_tx_to_cell(transaction);
        transaction
    }

    #[query]
    pub fn get_tx_from_log(idx: u64) -> Option<BoundedTransaction> {
        Service::get_tx_from_log(idx)
    }

    #[update]
    pub async fn push_tx_to_log(transaction: BoundedTransaction) -> u64 {
        Service::push_tx_to_log(transaction)
    }

    #[query]
    pub fn get_tx_from_unboundedmap(key: u64) -> Option<UnboundedTransaction> {
        Service::get_tx_from_unboundedmap(key)
    }

    #[update]
    pub async fn insert_tx_to_unboundedmap(transaction: UnboundedTransaction) -> u64 {
        Service::insert_tx_to_unboundedmap(transaction)
    }

    #[query]
    pub fn get_tx_from_multimap(key: u64) -> Option<BoundedTransaction> {
        Service::get_tx_from_multimap(key)
    }

    #[update]
    pub async fn insert_tx_to_multimap(transaction: BoundedTransaction) -> u64 {
        Service::insert_tx_to_multimap(transaction)
    }

    #[query]
    pub fn get_tx_from_vec(idx: u64) -> Option<BoundedTransaction> {
        Service::get_tx_from_vec(idx)
    }

    #[update]
    pub async fn push_tx_to_vec(transaction: BoundedTransaction) -> u64 {
        Service::push_tx_to_vec(transaction)
    }

    #[query]
    pub fn get_tx_from_ring_buffer(idx: u64) -> Option<BoundedTransaction> {
        Service::get_tx_from_ring_buffer(idx)
    }

    #[update]
    pub async fn push_tx_to_ring_buffer(transaction: BoundedTransaction) -> u64 {
        Service::push_tx_to_ring_buffer(transaction)
    }

