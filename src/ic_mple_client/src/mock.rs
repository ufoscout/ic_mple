use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use candid::utils::ArgumentEncoder;
use candid::{CandidType, Decode, Encode};
use serde::de::DeserializeOwned;

use crate::client::CanisterClient;
use crate::CanisterClientResult;

/// This client is used to mock the IC canister behavior in tests.
#[derive(Default, Clone)]
pub struct MockCanisterClient {
    queries: Arc<Mutex<HashMap<String, VecDeque<Box<dyn FnOnce() -> CanisterClientResult<Vec<u8>> + Send>>>>>,
    updates: Arc<Mutex<HashMap<String, VecDeque<Box<dyn FnOnce() -> CanisterClientResult<Vec<u8>> + Send>>>>>,
}

impl MockCanisterClient {
    

    /// Adds a query response to the mock client.
    ///
    /// The method and response are associated together in the mock client, so
    /// that when the mock client is queried with the given method, it will
    /// return the associated response.
    /// Once used, the response will be removed from the mock client.
    ///
    /// # Parameters
    ///
    /// - `method`: The method to associate with the response.
    /// - `response`: The response to associate with the method.
    pub fn add_query<R>(&self, method: &str, response: CanisterClientResult<R>)
    where
        R: DeserializeOwned + CandidType + Send,
    {
        match response {
            Ok(response) => {
                let response = Encode!(&response).unwrap();
                self.add_query_fn(method, Box::new(move || Ok(response.clone())));
            },
            Err(err) => {
                self.add_query_fn(method, Box::new(move || Err(err)));
            },
        }
    }

    
    /// Adds a query response to the mock client.
    ///
    /// The method and response are associated together in the mock client, so
    /// that when the mock client is queried with the given method, it will
    /// return the associated response.
    /// Once used, the response will be removed from the mock client.
    ///
    /// # Parameters
    ///
    /// - `method`: The method to associate with the response.
    /// - `response`: The response to associate with the method.
    pub fn add_query_fn(&self, method: &str, response: Box<dyn FnOnce() -> CanisterClientResult<Vec<u8>> + Send>) {
        let mut queries = self.queries.lock().unwrap();
        queries.entry(method.to_string()).or_default().push_back(response);
    }

    /// Adds an update response to the mock client.
    ///
    /// The method and response are associated together in the mock client, so
    /// that when the mock client is updated with the given method, it will
    /// return the associated response.
    /// Once used, the response will be removed from the mock client.
    ///
    /// # Parameters
    ///
    /// - `method`: The method to associate with the response.
    /// - `response`: The response to associate with the method.
    pub fn add_update<R>(&self, method: &str, response: CanisterClientResult<R>)
    where
        R: DeserializeOwned + CandidType + Send,
    {
        match response {
            Ok(response) => {
                let response = Encode!(&response).unwrap();
                self.add_update_fn(method, Box::new(move || Ok(response.clone())));
            },
            Err(err) => {
                self.add_update_fn(method, Box::new(move || Err(err)));
            },
        }
    }

    /// Adds an update response to the mock client.
    ///
    /// The method and response are associated together in the mock client, so
    /// that when the mock client is updated with the given method, it will
    /// return the associated response.
    /// Once used, the response will be removed from the mock client.
    ///
    /// # Parameters
    ///
    /// - `method`: The method to associate with the response.
    /// - `response`: The response to associate with the method.
    pub fn add_update_fn(&self, method: &str, response: Box<dyn FnOnce() -> CanisterClientResult<Vec<u8>> + Send>) {
        let mut updates = self.updates.lock().unwrap();
        updates.entry(method.to_string()).or_default().push_back(response);
    }

    /// Clears all query and update responses from the mock client.
    pub fn clear(&self) {
        let mut queries = self.queries.lock().unwrap();
        queries.clear();
        let mut updates = self.updates.lock().unwrap();
        updates.clear();
    }

}

impl CanisterClient for MockCanisterClient {

    async fn update<T, R>(&self, method: &str, _args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType + Send,
    {
        let mut updates = self.updates.lock().unwrap();
        let update = updates.get_mut(method).and_then(|v| v.pop_front()).expect("No response for update call [{method}] in mock client");
        let response = update();
         match response {
             Ok(response) => {
                 let decoded = Decode!(&response, R).expect("The mock client response for update call [{method}] cannot be decoded to the expected type");
                 Ok(decoded)
             },
             Err(err) => Err(err),
         }
    }

    async fn query<T, R>(&self, method: &str, _args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType + Send,
    {
        let mut queries = self.queries.lock().unwrap();
        let query = queries.get_mut(method).and_then(|v| v.pop_front()).expect("No response for query call [{method}] in mock client");
        let response = query();
         match response {
             Ok(response) => {
                 let decoded = Decode!(&response, R).expect("The mock client response for query call [{method}] cannot be decoded to the expected type");
                 Ok(decoded)
             },
             Err(err) => Err(err),
         }
    }

}

#[cfg(test)]
mod tests {

    use serde::Deserialize;

    use super::*;

    #[derive(CandidType, Deserialize, Debug)]
    struct TestCandidType {
        value: u64,
    }

    #[tokio::test]
    async fn test_mock_client() {
        let mock_client = MockCanisterClient::default();

        mock_client.add_query("query", Ok(42u64));
        mock_client.add_update("update", Ok(45u64));
        mock_client.add_update("update", Ok("hello".to_string()));
        mock_client.add_query("query", Ok(TestCandidType { value: 100u64 }));

        assert_eq!(mock_client.update::<_, u64>("update", ()).await.unwrap(), 45);
        assert_eq!(mock_client.query::<_, u64>("query", ()).await.unwrap(), 42);
        assert_eq!(mock_client.query::<_, TestCandidType>("query", ()).await.unwrap().value, 100);
        assert_eq!(mock_client.update::<_, String>("update", ()).await.unwrap(), "hello");

    }

    /// Test that the mock client can be cloned
    #[tokio::test]
    async fn test_mock_client_clone() {
        let mock_client = MockCanisterClient::default();

        mock_client.add_query("query", Ok(42u64));
        mock_client.add_update("update", Ok(45u64));
        mock_client.add_update("update", Ok("hello".to_string()));
        mock_client.add_query("query", Ok(TestCandidType { value: 100u64 }));

        let mock_client2 = mock_client.clone(); 

        assert_eq!(mock_client.update::<_, u64>("update", ()).await.unwrap(), 45);
        assert_eq!(mock_client.query::<_, u64>("query", ()).await.unwrap(), 42);
        assert_eq!(mock_client2.query::<_, TestCandidType>("query", ()).await.unwrap().value, 100);
        assert_eq!(mock_client2.update::<_, String>("update", ()).await.unwrap(), "hello");
    }

    /// Test that the mock client panics if a query response is used more than once
    #[tokio::test]
    #[should_panic]
    async fn test_mock_client_query_panic() {
        let mock_client = MockCanisterClient::default();    
        mock_client.add_query("query", Ok(42u64));
        mock_client.query::<_, u64>("query", ()).await.unwrap();
        mock_client.query::<_, u64>("query", ()).await.unwrap();
    }   

    /// Test that the mock client panics if an update response is used more than once
    #[tokio::test]
    #[should_panic]
    async fn test_mock_client_update_panic() {
        let mock_client = MockCanisterClient::default();    
        mock_client.add_update("update", Ok(42u64));
        mock_client.update::<_, u64>("update", ()).await.unwrap();
        mock_client.update::<_, u64>("update", ()).await.unwrap();
    }

    #[test]
    fn test_mock_client_clear() {
        let mock_client = MockCanisterClient::default();    
        mock_client.add_query("query", Ok(42u64));
        mock_client.add_update("update", Ok(42u64));
        mock_client.clear();
        assert!(mock_client.queries.lock().unwrap().is_empty());
        assert!(mock_client.updates.lock().unwrap().is_empty());
    }

}