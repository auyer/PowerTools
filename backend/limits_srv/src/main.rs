use std::sync::atomic::{Ordering, AtomicU64, AtomicI64};
use std::sync::{RwLock, Arc};

use serde::Serialize;
use warp::Filter;

static VISIT_V1_COUNT: AtomicU64 = AtomicU64::new(0);
static VISIT_V2_COUNT: AtomicU64 = AtomicU64::new(0);

static START_TIME: AtomicI64 = AtomicI64::new(0);

fn get_limits_v1(base: &limits_core::json::Base) -> impl warp::Reply {
    VISIT_V1_COUNT.fetch_add(1, Ordering::AcqRel);
    //println!("Limits got");
    warp::reply::json(base)
}

fn get_limits_v2(base: &limits_core::json_v2::Base) -> impl warp::Reply {
    VISIT_V2_COUNT.fetch_add(1, Ordering::AcqRel);
    //println!("Limits got");
    warp::reply::json(base)
}

#[derive(Serialize)]
struct Visits {
    visits_v1: u64,
    visits_v2: u64,
    since: i64, // Unix time (since epoch)
}

fn get_visits() -> impl warp::Reply {
    let count_v1 = VISIT_V1_COUNT.load(Ordering::Relaxed);
    let count_v2 = VISIT_V2_COUNT.load(Ordering::Relaxed);
    let start = START_TIME.load(Ordering::Relaxed);
    //println!("Count got");
    warp::reply::json(&Visits {
        visits_v1: count_v1,
        visits_v2: count_v2,
        since: start,
    })
}

#[allow(opaque_hidden_inferred_bound)]
fn routes(base: Arc<RwLock<limits_core::json::Base>>, base2: Arc<RwLock<limits_core::json_v2::Base>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(
        warp::path!("powertools" / "v1")
            .map(move || {
                let base = base.read().expect("Failed to acquire base limits read lock");
                get_limits_v1(&base)
            })
        .or(
            warp::path!("powertools" / "count")
                .map(get_visits)
        )
        .or(
            warp::path!("powertools" / "v2")
            .map(move || {
                let base2 = base2.read().expect("Failed to acquire base limits read lock");
                get_limits_v2(&base2)
            })
        )
    ).recover(recovery)
}

pub async fn recovery(reject: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if reject.is_not_found() {
        Ok(warp::hyper::StatusCode::NOT_FOUND)
    } else {
        Err(reject)
    }
}

#[tokio::main]
async fn main() {
    START_TIME.store(chrono::Utc::now().timestamp(), Ordering::Relaxed);
    let file = std::fs::File::open("./pt_limits.json").expect("Failed to read limits file");
    let limits: limits_core::json::Base = serde_json::from_reader(file).expect("Failed to parse limits file");
    assert!(limits.refresh.is_some(), "`refresh` cannot be null, since it will brick future refreshes");

    let file = std::fs::File::open("./pt_limits_v2.json").expect("Failed to read limits file");
    let limits_v2: limits_core::json_v2::Base = serde_json::from_reader(file).expect("Failed to parse limits file");
    assert!(limits_v2.refresh.is_some(), "`refresh` cannot be null, since it will brick future refreshes");

    warp::serve(routes(Arc::new(RwLock::new(limits)), Arc::new(RwLock::new(limits_v2))))
        .run(([0, 0, 0, 0], 8080))
        .await;
}

#[cfg(test)]
mod test {
    #[test]
    fn generate_default_pt_limits() {
        let limits = limits_core::json::Base::default();
        let output_file = std::fs::File::create("./pt_limits.json").unwrap();
        serde_json::to_writer_pretty(output_file, &limits).unwrap();
    }

    #[test]
    fn generate_default_pt_limits_v2() {
        let limits = limits_core::json_v2::Base::default();
        let output_file = std::fs::File::create("./pt_limits_v2.json").unwrap();
        serde_json::to_writer_pretty(output_file, &limits).unwrap();
    }
}
