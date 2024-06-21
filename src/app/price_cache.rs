use my_nosql_contracts::price_src::BidAskPriceSrc;
use rust_extensions::sorted_vec::SortedVecWithStrKey;
use tokio::sync::Mutex;

pub struct PriceCache {
    to_save: Mutex<SortedVecWithStrKey<BidAskPriceSrc>>,
}

impl PriceCache {
    pub fn new() -> Self {
        Self {
            to_save: Mutex::new(SortedVecWithStrKey::new()),
        }
    }

    pub async fn update(&self, items: impl Iterator<Item = BidAskPriceSrc>) {
        let mut data_access = self.to_save.lock().await;

        for item in items {
            data_access.insert_or_replace(item);
        }
    }

    pub async fn get_snapshot(&self) -> Vec<BidAskPriceSrc> {
        let mut data_access = self.to_save.lock().await;
        if data_access.len() == 0 {
            return Vec::new();
        }

        let mut result = SortedVecWithStrKey::new();

        std::mem::swap(&mut result, &mut *data_access);

        result.into_vec()
    }
}
