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
    tweets: HashMap<i32, Vec<(i32, i32)>>, // userId → [(timestamp, tweetId)]
    follows: HashMap<i32, HashSet<i32>>,   // userId → set of followeeIds
    time: i32,
}

impl Twitter {
    pub fn new() -> Self {
        Twitter {
            tweets: HashMap::new(),
            follows: HashMap::new(),
            time: 0,
        }
    }

    pub fn post_tweet(&mut self, user_id: i32, tweet_id: i32) {
        self.tweets
            .entry(user_id)
            .or_default()
            .push((self.time, tweet_id));
        self.time += 1;
    }

    pub fn get_news_feed(&self, user_id: i32) -> Vec<i32> {
        let mut heap: BinaryHeap<(i32, i32, usize, i32)> = BinaryHeap::new();
        // (timestamp, tweetId, index_in_user_tweets, userId)

        // Get all relevant users (self + followees)
        let mut users: HashSet<i32> = HashSet::new();
        users.insert(user_id);
        if let Some(followees) = self.follows.get(&user_id) {
            users.extend(followees);
        }

        // Push the most recent tweet from each user
        for &uid in &users {
            if let Some(tweets) = self.tweets.get(&uid) {
                if let Some(&(ts, tid)) = tweets.last() {
                    heap.push((ts, tid, tweets.len() - 1, uid));
                }
            }
        }

        // Collect up to 10 most recent
        let mut result = Vec::new();
        while result.len() < 10 && !heap.is_empty() {
            let (ts, tid, idx, uid) = heap.pop().unwrap();
            result.push(tid);
            if idx > 0 {
                if let Some(tweets) = self.tweets.get(&uid) {
                    let (prev_ts, prev_tid) = tweets[idx - 1];
                    heap.push((prev_ts, prev_tid, idx - 1, uid));
                }
            }
        }

        result
    }

    pub fn follow(&mut self, follower_id: i32, followee_id: i32) {
        self.follows
            .entry(follower_id)
            .or_default()
            .insert(followee_id);
    }

    pub fn unfollow(&mut self, follower_id: i32, followee_id: i32) {
        self.follows
            .entry(follower_id)
            .or_default()
            .remove(&followee_id);
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
}
