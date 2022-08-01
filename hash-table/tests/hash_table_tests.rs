use bytemuck::Pod;
use bytemuck::Zeroable;
use hash_table::*;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use rand::{self, Rng};
use std::collections::BTreeMap;

const MAX_SIZE: usize = 50000;
const NUM_BUCKETS: usize = MAX_SIZE / 4;

#[repr(C)]
#[derive(Default, Copy, Clone, PartialEq)]
struct Widget {
    a: u128,
    b: u128,
    size: u64,
}

unsafe impl Zeroable for Widget {}
unsafe impl Pod for Widget {}

impl Widget {
    pub fn new_random(r: &mut ThreadRng) -> Self {
        Self {
            a: r.gen::<u128>(),
            b: r.gen::<u128>(),
            size: r.gen::<u64>(),
        }
    }
}

#[tokio::test(threaded_scheduler)]
async fn test_simulate() {
    type HashMap = HashTable<u128, Widget, NUM_BUCKETS, MAX_SIZE>;
    let mut buf = vec![0u8; std::mem::size_of::<HashMap>()];
    let hm = HashMap::new_from_slice(buf.as_mut_slice());
    println!("Size: {}", std::mem::size_of::<HashMap>());
    let mut rng = thread_rng();
    let mut keys = vec![];
    let mut map = Box::new(BTreeMap::new());
    let mut s = 0;
    let mut v;
    for _ in 0..(MAX_SIZE - 1) {
        let k = rng.gen::<u128>();
        v = Widget::new_random(&mut rng);
        assert!(hm.insert(k, v) != None);
        s += 1;
        assert!(s == hm.size());
        map.insert(k, v);
        keys.push(k);
    }

    let k = rng.gen::<u128>();
    let v = Widget::new_random(&mut rng);
    assert!(hm.insert(k, v)  == None);

    for k in keys.iter() {
        assert!(hm.remove(k) != None);
        s -= 1;
        map.remove(k);
    }
    keys = vec![];

    for _i in 0..(MAX_SIZE >> 1) {
        let k = rng.gen::<u128>();
        let v = Widget::new_random(&mut rng);
        assert!(hm.insert(k, v) != None);
        s += 1;
        map.insert(k, v);
        keys.push(k);
    }

    for _ in 0..100000 {
        println!("{} {}", s, hm.size());
        assert!(s == hm.size());
        let sample = rng.gen::<f64>();
        if sample < 0.33 {
            if hm.size() >= MAX_SIZE - 1 {
                continue;
            }
            println!("I");
            let k = rng.gen::<u128>();
            let v = Widget::new_random(&mut rng);
            assert!(hm.insert(k, v) != None);
            s += 1;
            map.insert(k, v);
            keys.push(k);
        } else if sample < 0.66 {
            if keys.is_empty() {
                continue;
            }
            println!("R");
            let j = rng.gen_range(0, keys.len());
            let key = keys[j];
            keys.swap_remove(j);
            assert!(hm[&key] == map[&key]);
            assert!(hm.remove(&key) != None);
            map.remove(&key);
            s -= 1;
        } else {
            if keys.is_empty() {
                continue;
            }
            println!("U");
            let j = rng.gen_range(0, keys.len());
            let key = keys[j];
            assert!(hm.contains(&key));
            let v = Widget::new_random(&mut rng);
            assert!(hm.insert(key, v) != None);
            map.insert(key, v);
        }
    }
    // let nodes = hm.inorder_traversal();
    // for ((k1, v1), (k2, v2)) in map.iter().zip(nodes.iter()) {
    //     assert!(*k1 == *k2);
    //     assert!(*v1 == *v2);
    // }
}
