// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

pub struct TimeSeries {
    internal_time_series: Vec<u64>,
    internal_head_pointer: usize,
}

impl TimeSeries {
    pub fn new(length: usize) -> TimeSeries {
        let zeroed_ts: Vec<u64> = vec![0; length];

        TimeSeries {
            internal_time_series: zeroed_ts,
            internal_head_pointer: 0,
        }
    }

    pub fn length(&self) -> usize {
        self.internal_time_series.len()
    }

    pub fn append(&mut self, value: u64) {
        self.internal_time_series[self.internal_head_pointer] = value;
        self.internal_head_pointer = (self.internal_head_pointer + 1) % self.length();
    }

    pub fn as_array(&self) -> Vec<u64> {
        let len = self.length();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let index = (self.internal_head_pointer + i) % len;
            result.push(self.internal_time_series[index]);
        }

        result
    }
}






#[test]
fn some_test() {
    let mut ts = TimeSeries::new(5);

    ts.append(10);
    ts.append(20);
    ts.append(30);
    ts.append(40);
    ts.append(50);

    println!("{:?}", ts.as_array());

    ts.append(60);
    println!("{:?}", ts.as_array());
}
