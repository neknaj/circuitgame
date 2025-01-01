use rand::Rng;
use std::collections::HashMap;

pub struct ResourceManager<T> {
    resources: Vec<T>,
    id_map: HashMap<u32, usize>, // 外部ID -> 内部インデックス
    reverse_map: HashMap<usize, u32>, // 内部インデックス -> 外部ID
}

impl<T> ResourceManager<T> {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            id_map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    pub fn add_resource(&mut self, resource: T) -> u32 {
        let index = self.resources.len();
        self.resources.push(resource);

        // ランダムなIDを生成（u32型）
        let mut id;
        loop {
            id = rand::random::<u32>(); // rand::randomを使ってu32型のランダムなIDを生成
            if !self.id_map.contains_key(&id) {
                break;
            }
        }

        // マッピングを保存
        self.id_map.insert(id, index);
        self.reverse_map.insert(index, id);

        id
    }


    pub fn get_resource(&mut self, id: u32) -> Option<&mut T> {
        self.id_map.get(&id).and_then(|&index| self.resources.get_mut(index))
    }

    pub fn remove_resource(&mut self, id: u32) -> Option<T> {
        if let Some(index) = self.id_map.remove(&id) {
            self.reverse_map.remove(&index);
            Some(self.resources.remove(index))
        } else {
            None
        }
    }
}
