//! 经验回放缓冲区的单元测试

use rlkit::replay_buffer::ReplayBuffer;
use rlkit::types::{Sample, Status, Reward};
use rlkit::Action;

#[test]
fn test_new_buffer() {
    // 测试创建新的缓冲区
    let buffer: ReplayBuffer<f32, f32> = ReplayBuffer::new(100);
    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());
}

#[test]
fn test_push_and_len() {
    // 测试添加样本和获取长度
    let mut buffer: ReplayBuffer<f32, f32> = ReplayBuffer::new(100);
    
    // 创建一些测试样本
    let sample1 = Sample::<f32, f32> {
        state: Status::new(vec![1.0, 2.0, 3.0], vec![10.0, 10.0, 10.0]),
        action: Action::new(vec![0.0], vec![10.0]),
        reward: Reward(1.0),
        next_state: Status::new(vec![4.0, 5.0, 6.0], vec![10.0, 10.0, 10.0]),
        done: false,
    };
    
    let sample2 = Sample::<f32, f32> {
        state: Status::new(vec![7.0, 8.0, 9.0], vec![10.0, 10.0, 10.0]),
        action: Action::new(vec![1.0], vec![10.0]),
        reward: Reward(2.0),
        next_state: Status::new(vec![10.0, 11.0, 12.0], vec![10.0, 10.0, 10.0]),
        done: true,
    };
    
    // 添加样本
    buffer.push(sample1.clone());
    assert_eq!(buffer.len(), 1);
    assert!(!buffer.is_empty());
    
    buffer.push(sample2.clone());
    assert_eq!(buffer.len(), 2);
}

#[test]
fn test_buffer_capacity() {
    // 测试缓冲区容量限制
    let mut buffer: ReplayBuffer<f32, f32> = ReplayBuffer::new(3);
    
    // 添加超过容量的样本
    for i in 0..5 {
        let sample = Sample::<f32, f32> {
              state: Status::new(vec![i as f32], vec![10.0]),
              action: Action::new(vec![i as f32], vec![10.0]),
              reward: Reward(i as f32),
              next_state: Status::new(vec![(i + 1) as f32], vec![10.0]),
              done: i == 4,
        };
        buffer.push(sample);
    }
    
    // 缓冲区应该只保留最新的3个样本
    assert_eq!(buffer.len(), 3);
}

#[test]
fn test_sample() {
    // 测试采样功能
    let mut buffer: ReplayBuffer<f32, f32> = ReplayBuffer::new(100);
    
    // 添加一些测试样本
    for i in 0..10 {
        let sample = Sample::<f32, f32> {
              state: Status::new(vec![i as f32], vec![10.0]),
              action: Action::new(vec![i as f32], vec![10.0]),
              reward: Reward(i as f32),
              next_state: Status::new(vec![(i + 1) as f32], vec![10.0]),
              done: i == 9,
        };
        buffer.push(sample);
    }
    
    // 测试空批量大小
    let empty_batch = buffer.sample(0);
    assert!(empty_batch.is_empty());
    
    // 测试正常批量大小
    let batch_size = 5;
    let samples = buffer.sample(batch_size);
    assert_eq!(samples.len(), batch_size);
    
    // 验证所有采样的样本都是有效的
    for sample in samples {
        assert!(!sample.state.as_slice().is_empty());
        assert!(!sample.next_state.as_slice().is_empty());  
    }
}

#[test]
fn test_sample_empty_buffer() {
    // 测试从空缓冲区采样
    let buffer: ReplayBuffer<f32, f32> = ReplayBuffer::new(100);
    let samples = buffer.sample(5);
    assert!(samples.is_empty());
}

#[test]
fn test_sample_cloning() {
    // 测试采样是否正确克隆样本
    let mut buffer: ReplayBuffer<f32, f32> = ReplayBuffer::new(100);
    
    let original_sample = Sample::<f32, f32> {
          state: Status::new(vec![1.0, 2.0, 3.0], vec![10.0, 10.0, 10.0]),
          action: Action::new(vec![0.0], vec![10.0]),
          reward: Reward(1.0),
          next_state: Status::new(vec![4.0, 5.0, 6.0], vec![10.0, 10.0, 10.0]),
              done: false,
    };
    
    buffer.push(original_sample.clone());
    
    // 采样并修改采样的样本
    let mut samples = buffer.sample(1);
    let mut sampled_sample = samples.pop().unwrap();
    sampled_sample.reward = Reward(100.0);
    
    // 原始样本应该保持不变
    let samples_after_modification = buffer.sample(1);
    assert_eq!(samples_after_modification[0].reward, original_sample.reward);
}