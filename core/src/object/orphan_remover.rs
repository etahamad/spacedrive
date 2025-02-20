use crate::prisma::{object, tag_on_object, PrismaClient};

use std::{sync::Arc, time::Duration};

use tokio::{
	select,
	sync::mpsc,
	time::{interval_at, Instant, MissedTickBehavior},
};
use tracing::{error, trace};

const TEN_SECONDS: Duration = Duration::from_secs(10);
const ONE_MINUTE: Duration = Duration::from_secs(60);

// Actor that can be invoked to find and delete objects with no matching file paths
#[derive(Clone)]
pub struct OrphanRemoverActor {
	tx: mpsc::Sender<()>,
}

impl OrphanRemoverActor {
	pub fn spawn(db: Arc<PrismaClient>) -> Self {
		let (tx, mut rx) = mpsc::channel(4);

		tokio::spawn(async move {
			let mut last_checked = Instant::now();

			let mut check_interval = interval_at(Instant::now() + ONE_MINUTE, ONE_MINUTE);
			check_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

			loop {
				// Here we wait for a signal or for the tick interval to be reached
				select! {
					_ =  check_interval.tick() => {}
					signal = rx.recv() => {
						if signal.is_none() {
							break;
						}
					}
				}

				// For any of them we process a clean up if a time since the last one already passed
				if last_checked.elapsed() > TEN_SECONDS {
					Self::process_clean_up(&db).await;
					last_checked = Instant::now();
				}
			}
		});

		Self { tx }
	}

	pub async fn invoke(&self) {
		self.tx.send(()).await.ok();
	}

	async fn process_clean_up(db: &PrismaClient) {
		loop {
			let Ok(objects_ids) = db
				.object()
				.find_many(vec![object::file_paths::none(vec![])])
				.take(512)
				.select(object::select!({ id }))
				.exec()
				.await
				.map(|objects| objects.into_iter()
					.map(|object| object.id)
					.collect::<Vec<_>>()
				)
				.map_err(|e| error!("Failed to fetch orphaned objects: {e:#?}"))
				else {
				break;
			};

			if objects_ids.is_empty() {
				break;
			}

			trace!("Removing {} orphaned objects", objects_ids.len());

			if let Err(e) = db
				._batch((
					db.tag_on_object()
						.delete_many(vec![tag_on_object::object_id::in_vec(objects_ids.clone())]),
					db.object()
						.delete_many(vec![object::id::in_vec(objects_ids)]),
				))
				.await
			{
				error!("Failed to remove orphaned objects: {e:#?}");
			}
		}
	}
}
