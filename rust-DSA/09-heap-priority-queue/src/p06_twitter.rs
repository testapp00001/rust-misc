// ============================================================================
// Problem: Design Twitter (LeetCode #355)
// ============================================================================
// Design a simplified Twitter where users can:
// - postTweet(userId, tweetId)
// - getNewsFeed(userId) — 10 most recent tweets from user + followees
// - follow(followerId, followeeId)
// - unfollow(followerId, followeeId)
//
// ============================================================================
// APPROACH: HashMap + Heap (O(n log k) per getNewsFeed)
// ============================================================================
//
// Data structures:
// - tweets: HashMap<userId, Vec<(timestamp, tweetId)>>
// - follows: HashMap<userId, HashSet<userId>>
//
// getNewsFeed: Merge recent tweets from user + followees using a heap.
// ============================================================================


use std::collections::{BinaryHeap, HashMap, HashSet};

pub struct Twitter {
    // TODO: Define fields
}

impl Twitter {
    pub fn new() -> Self {
        todo!("Implement new")
    }

    pub fn post_tweet(&mut self, user_id: i32, tweet_id: i32) {
        todo!("Implement post_tweet")
    }

    pub fn get_news_feed(&self, user_id: i32) -> Vec<i32> {
        todo!("Implement get_news_feed")
    }

    pub fn follow(&mut self, follower_id: i32, followee_id: i32) {
        todo!("Implement follow")
    }

    pub fn unfollow(&mut self, follower_id: i32, followee_id: i32) {
        todo!("Implement unfollow")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut twitter = Twitter::new();
        twitter.post_tweet(1, 5);
        assert_eq!(twitter.get_news_feed(1), vec![5]);
        twitter.follow(1, 2);
        twitter.post_tweet(2, 6);
        assert_eq!(twitter.get_news_feed(1), vec![6, 5]);
        twitter.unfollow(1, 2);
        assert_eq!(twitter.get_news_feed(1), vec![5]);
    }


    #[test]
    #[ignore]
    fn bench_performance() {
        // ⏱️  Benchmark test
        // Run: cargo test -p <package> bench_performance -- --ignored --nocapture
        //
        // To use: uncomment and customize the code below with your function
        // and realistic test data.
        //
        // let iterations = 10_000;
        // let input = /* generate your test input here */;
        // let start = std::time::Instant::now();
        // for _ in 0..iterations {
        //     let _ = new(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}