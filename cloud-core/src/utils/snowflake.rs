use std::time::{SystemTime, UNIX_EPOCH};

pub struct SnowFlake {
    sequence: i64,
    worker_id: i64,
    datacenter_id: i64,

    start_timestamp: i64,

    sequence_bits: i64,
    worker_id_bits: i64,
    datacenter_id_bits: i64,

    max_sequence: i64,
    max_worker_id: i64,
    max_datacenter_id: i64,

    worker_id_shift: i64,
    datacenter_id_shift: i64,
    timestamp_shift: i64,

    last_timestamp: i64,
}

impl SnowFlake {
    pub fn new(worker_id: i64, datacenter_id: i64) -> Self {
        // 2019-01-01 00:00:00  1546272000000
        let start_timestamp = 1546272000000;
        let sequence_bits = 12;
        let worker_id_bits = 5;
        let datacenter_id_bits = 5;

        let max_sequence = -1 ^ (-1 << sequence_bits);
        let max_worker_id = -1 ^ (-1 << worker_id_bits);
        let max_datacenter_id = -1 ^ (-1 << datacenter_id_bits);

        let worker_id_shift = sequence_bits;
        let datacenter_id_shift = sequence_bits + worker_id_bits;
        let timestamp_shift = sequence_bits + worker_id_bits + datacenter_id_bits;

        if worker_id > max_worker_id || worker_id < 0 {
            panic!(
                "worker_id can't be greater than {} or less than 0 ",
                max_worker_id
            );
        }

        if datacenter_id > max_datacenter_id || datacenter_id < 0 {
            panic!(
                "datacenter_id can't be greater than {} or less than 0 ",
                max_datacenter_id
            );
        }

        SnowFlake {
            sequence: 0,
            worker_id: 1,
            datacenter_id: 1,
            last_timestamp: -1,

            start_timestamp,
            sequence_bits,
            worker_id_bits,
            datacenter_id_bits,

            max_sequence,
            max_worker_id,
            max_datacenter_id,

            worker_id_shift,
            datacenter_id_shift,
            timestamp_shift,
        }
    }

    pub fn next_id(&mut self) -> i64 {
        let mut timestamp = self.get_timestamp();
        if timestamp < self.last_timestamp {
            panic!(
                "Clock moved backwards.  Refusing to generate id for {} milliseconds",
                self.last_timestamp - timestamp
            );
        }

        if self.last_timestamp == timestamp {
            // 相同毫秒内，序列号自增
            self.sequence = (self.sequence + 1) & self.max_sequence;
            // 同一毫秒的序列数已经达到最大
            if self.sequence == 0 {
                timestamp = self.til_next_millis(self.last_timestamp);
            }
        } else {
            // 不同毫秒内，序列号置为0
            self.sequence = 0;
        }

        self.last_timestamp = timestamp;

        // 时间戳部分 | 数据中心部分 | 机器标识部分 | 序列号部分
        ((timestamp - self.start_timestamp) << self.timestamp_shift)
            | (self.datacenter_id << self.datacenter_id_shift)
            | (self.worker_id << self.worker_id_shift)
            | self.sequence
    }

    fn get_timestamp(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    fn til_next_millis(&self, last_timestamp: i64) -> i64 {
        let mut timestamp = self.get_timestamp();
        while timestamp <= last_timestamp {
            timestamp = self.get_timestamp();
        }
        timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snowflake() {
        let mut snowflake = SnowFlake::new(1, 1);
        let id = snowflake.next_id();
        println!("{}", id);
    }

    #[test]
    fn test_snowflake_single() {
        use std::collections::HashSet;

        //let mut s_ids = HashSet::new();
        let mut s = SnowFlake::new(1, 1);
        let start = std::time::Instant::now();
        for _ in 0..100000 {
            let id = s.next_id();
            //s_ids.insert(id);
        }
        let end = std::time::Instant::now();
        //println!("Done: {:?}, id count: {:}", end - start, s_ids.len());
        println!("Done: {:?}, id count: ", end - start);
    }

    #[test]
    fn test_snowflake_multi() {
        use std::sync::{Arc, Mutex};
        use std::sync::atomic::{AtomicUsize, Ordering};

        let start = std::time::Instant::now();
        let global_thread_count = AtomicUsize::new(0);
        let snowflake = SnowFlake::new(1, 1);
        let s = Arc::new(Mutex::new(snowflake));
        for _ in 0..100000 {
            let s1 = Arc::clone(&s);
            global_thread_count.fetch_add(1, Ordering::SeqCst);
            std::thread::spawn(move || {
                let _ = s1.lock().unwrap().next_id();
                //println!("{}", s2);
            });
            global_thread_count.fetch_sub(1, Ordering::SeqCst);
        }

        while global_thread_count.load(Ordering::SeqCst) != 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        let end = std::time::Instant::now();
        println!("Done: {:?}", end - start);
    }
}
