use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use warp::Filter;

#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::StatusCode;
    use warp::test::request;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_sync_point() {
        let state = Arc::new(Mutex::new(HashMap::new()));
        let state_filter = warp::any().map(move || Arc::clone(&state));

        let sync_route = warp::path!("wait-for-second-party" / String)
            .and(state_filter.clone())
            .and(warp::post())
            .and_then(handle_sync);

        let response1 = tokio::spawn({
            let sync_route = sync_route.clone();
            async move {
                request()
                    .method("POST")
                    .path("/wait-for-second-party/test-id")
                    .reply(&sync_route)
                    .await
            }
        });

        // Give a little delay to ensure the first request is waiting
        sleep(Duration::from_millis(100)).await;

        let response2 = tokio::spawn({
            let sync_route = sync_route.clone();
            async move {
                request()
                    .method("POST")
                    .path("/wait-for-second-party/test-id")
                    .reply(&sync_route)
                    .await
            }
        });

        let response1 = response1.await.unwrap();
        let response2 = response2.await.unwrap();

        assert_eq!(response1.status(), StatusCode::OK);
        assert_eq!(response1.body(), "Synced");

        assert_eq!(response2.status(), StatusCode::OK);
        assert_eq!(response2.body(), "Synced");
    }

    #[tokio::test]
    async fn test_timeout() {
        let state = Arc::new(Mutex::new(HashMap::new()));
        let state_filter = warp::any().map(move || Arc::clone(&state));

        let sync_route = warp::path!("wait-for-second-party" / String)
            .and(state_filter.clone())
            .and(warp::post())
            .and_then(handle_sync);

        let response1 = tokio::spawn({
            let sync_route = sync_route.clone();
            async move {
                request()
                    .method("POST")
                    .path("/wait-for-second-party/timeout-id")
                    .reply(&sync_route)
                    .await
            }
        });

        // Wait longer than the timeout duration
        sleep(Duration::from_secs(11)).await;

        let response1 = response1.await.unwrap();

        assert_eq!(response1.status(), StatusCode::REQUEST_TIMEOUT);
        assert_eq!(response1.body(), "Timeout");
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(HashMap::new()));
    let state_filter = warp::any().map(move || Arc::clone(&state));

    let sync_route = warp::path!("wait-for-second-party" / String)
        .and(state_filter.clone())
        .and(warp::post())
        .and_then(handle_sync);

    warp::serve(sync_route).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_sync(
    unique_id: String,
    state: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<()>>>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    {
        let mut state_lock = state.lock().unwrap();
        if let Some(tx_existing) = state_lock.remove(&unique_id) {
            tx_existing.send(()).unwrap_or_default();
            return Ok(warp::reply::with_status("Synced", warp::http::StatusCode::OK));
        } else {
            state_lock.insert(unique_id.clone(), tx);
        }
    }

    let timeout = tokio::time::sleep(Duration::from_secs(10));
    tokio::select! {
        _ = rx => {
            Ok(warp::reply::with_status("Synced", warp::http::StatusCode::OK))
        },
        _ = timeout => {
            let mut state_lock = state.lock().unwrap();
            state_lock.remove(&unique_id);
            Ok(warp::reply::with_status("Timeout", warp::http::StatusCode::REQUEST_TIMEOUT))
        },
    }
}
