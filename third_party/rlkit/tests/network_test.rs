//! Q网络的单元测试

use candle_core::{Device, Tensor, Result, Module};
use rlkit::network::NeuralNetwork;

#[test]
fn test_qnetwork_creation() -> Result<()> {
    // 创建CPU设备
    let device = Device::Cpu;
    
    // 创建一个简单的Q网络：输入维度4，隐藏层[64, 32]，输出维度2
    let input_dim = 4;
    let hidden_dims = &[64, 32];
    let output_dim = 2;
    
    let network = NeuralNetwork::new(input_dim, hidden_dims, output_dim, &device)?;
    
    // 验证网络层的数量：隐藏层数量 + 输出层
    assert_eq!(network.hidden_dims().len(), hidden_dims.len());
    assert_eq!(network.output_dim(), output_dim);
    assert_eq!(network.input_dim(), input_dim);
    
    Ok(())
}

#[test]
fn test_qnetwork_forward_pass() -> Result<()> {
    let device = Device::Cpu;
    
    // 创建一个简单的Q网络
    let input_dim = 4;
    let hidden_dims = &[8, 4];
    let output_dim = 2;
    
    let network = NeuralNetwork::new(input_dim, hidden_dims, output_dim, &device)?;
    
    // 创建一个测试输入张量 [batch_size=2, input_dim=4]
    let input = Tensor::randn(0f32, 1f32, (2, input_dim), &device)?;
    
    // 执行前向传播
    let output = network.forward(&input)?;
    
    // 验证输出形状
    let output_shape = output.shape().dims();
    assert_eq!(output_shape.len(), 2);
    assert_eq!(output_shape[0], 2); // batch size
    assert_eq!(output_shape[1], output_dim); // output dimensions
    
    Ok(())
}

#[test]
fn test_qnetwork_parameters() -> Result<()> {
    let device = Device::Cpu;
    
    // 创建一个简单的Q网络
    let input_dim = 4;
    let hidden_dims = &[8, 4];
    let output_dim = 2;
    
    let network = NeuralNetwork::new(input_dim, hidden_dims, output_dim, &device)?;
    
    // 获取所有参数
    let params = network.parameters();
    
    // 每个线性层有2个参数（权重和偏置），共3个层，所以应该有6个参数
    assert_eq!(params.len(), 6);
    
    // 验证参数的维度
    // 第一个隐藏层: [8, 4]
    let shape_0 = params[0].shape().dims();
    assert_eq!(shape_0[0], 8);
    assert_eq!(shape_0[1], 4);
    
    let shape_1 = params[1].shape().dims();
    assert_eq!(shape_1[0], 8);
    
    // 第二个隐藏层: [4, 8]
    let shape_2 = params[2].shape().dims();
    assert_eq!(shape_2[0], 4);
    assert_eq!(shape_2[1], 8);
    
    let shape_3 = params[3].shape().dims();
    assert_eq!(shape_3[0], 4);
    
    // 输出层: [2, 4]
    let shape_4 = params[4].shape().dims();
    assert_eq!(shape_4[0], 2);
    assert_eq!(shape_4[1], 4);
    
    let shape_5 = params[5].shape().dims();
    assert_eq!(shape_5[0], 2);
    
    Ok(())
}

#[test]
fn test_qnetwork_save_and_load() -> Result<()> {
    use std::fs;
    
    let device = Device::Cpu;
    let input_dim = 4;
    let hidden_dims = &[8, 4];
    let output_dim = 2;
    
    // 创建并保存网络
    let network = NeuralNetwork::new(input_dim, hidden_dims, output_dim, &device)?;
    
    // 保存网络参数到临时文件
    let temp_path = "temp_qnetwork.safetensors";
    network.save(temp_path)?;
    
    // 加载网络
    let loaded_network = NeuralNetwork::load(temp_path, input_dim, hidden_dims, output_dim, &device)?;
    
    // 比较原始网络和加载网络的参数数量
    let original_params = network.parameters();
    let loaded_params = loaded_network.parameters();
    
    assert_eq!(original_params.len(), loaded_params.len());
    
    // 验证参数的形状是否相同
    for (orig, loaded) in original_params.iter().zip(loaded_params.iter()) {
        assert_eq!(orig.shape().dims(), loaded.shape().dims());
    }
    
    // 验证网络能够正常前向传播
    let test_input = Tensor::randn(0f32, 1f32, (1, input_dim), &device)?;
    
    // 确保原始网络和加载的网络都能正常前向传播，检查输出是否相同
    let original_output = network.forward(&test_input)?;
    let loaded_output = loaded_network.forward(&test_input)?;
    
    // 验证加载网络的输出形状正确
    let output_shape = loaded_output.shape().dims();
    assert_eq!(output_shape.len(), 2);
    assert_eq!(output_shape[0], 1);
    assert_eq!(output_shape[1], output_dim);
    
    // 比较原始网络和加载网络的输出是否相同
    // 由于浮点数计算的精度问题，我们使用近似比较
    let diff = original_output.sub(&loaded_output)?.abs()?.mean_all()?;
    println!("原始输出: {:?}", original_output.to_vec2::<f32>()?);
    println!("加载输出: {:?}", loaded_output.to_vec2::<f32>()?);
    let diff_value = diff.to_scalar::<f32>()?;
    
    // 对于完全相同的参数，输出应该非常接近（考虑到浮点数精度和保存/加载过程中的潜在差异）
    assert!(diff_value < 1e-5, "加载的模型输出与原始模型输出差异过大: {}", diff_value);
    
    // 清理临时文件
    fs::remove_file(temp_path).ok();
    
    Ok(())
}

#[test]
fn test_qnetwork_with_different_dimensions() -> Result<()> {
    let device = Device::Cpu;
    
    // 测试不同的网络配置
    let test_configs = [
        (10, &[128, 64, 32][..], 5),    // 3个隐藏层
        (2, &[16][..], 1),               // 1个隐藏层，单一输出
        (100, &[256, 128, 64, 32][..], 10), // 4个隐藏层，较宽的网络
    ];
    
    for (input_dim, hidden_dims, output_dim) in test_configs {
        // 创建网络
        let network = NeuralNetwork::new(input_dim, hidden_dims, output_dim, &device)?;
        
        // 创建测试输入
        let input = Tensor::randn(0f32, 1f32, (3, input_dim), &device)?;
        
        // 执行前向传播
        let output = network.forward(&input)?;
        
        // 验证输出形状
        let output_shape = output.shape().dims();
        assert_eq!(output_shape.len(), 2);
        assert_eq!(output_shape[0], 3);
        assert_eq!(output_shape[1], output_dim);
    }
    
    Ok(())
}

#[test]
fn test_qnetwork_training_fitting() -> Result<()> {
    use candle_nn::{optim::AdamW, Optimizer};
    
    let device = Device::Cpu;
    let input_dim = 1;
    let hidden_dims = &[64, 32];
    let output_dim = 1;
    
    // 创建网络
    let network = NeuralNetwork::new(input_dim, hidden_dims, output_dim, &device)?;
    
    // 创建优化器
    let mut optimizer = AdamW::new_lr(network.varmap.all_vars(), 1e-3)?;
    
    // 创建训练数据集：简单的二次函数 y = x^2
    let x_train = Tensor::randn(0f32, 1f32, (100, input_dim), &device)?;
    let y_train = x_train.powf(2.0)?;
    
    // 训练循环
    let epochs = 100;
    let mut best_loss = f32::INFINITY;
    
    for epoch in 0..epochs {
        // 使用整个数据集进行单次训练迭代
        let y_pred = network.forward(&x_train)?;
        
        // 计算损失（均方误差）
        let loss = y_pred.sub(&y_train)?.sqr()?.mean_all()?;
        let loss_value = loss.to_scalar::<f32>()?;
        
        // 反向传播和优化
        optimizer.backward_step(&loss)?;
        
        // 更新最佳损失
        if loss_value < best_loss {
            best_loss = loss_value;
        }
        
        // 每10个epoch打印一次
        if epoch % 10 == 0 {
            println!("Epoch {}/{}, Loss: {:.6}", epoch + 1, epochs, loss_value);
        }
    }
    
    // 验证网络拟合能力
    println!("最终训练损失: {:.6}", best_loss);
    
    // 测试几个具体的点
    let test_inputs = Tensor::new(&[[-1.0f32], [0.0], [1.0], [2.0]], &device)?;
    let expected_outputs = Tensor::new(&[[1.0f32], [0.0], [1.0], [4.0]], &device)?;
    let actual_outputs = network.forward(&test_inputs)?;
    
    println!("测试输入: {:?}", test_inputs.to_vec2::<f32>()?);
    println!("期望输出: {:?}", expected_outputs.to_vec2::<f32>()?);
    println!("实际输出: {:?}", actual_outputs.to_vec2::<f32>()?);
    
    // 计算测试损失
    let test_loss = actual_outputs.sub(&expected_outputs)?.sqr()?.mean_all()?;
    let test_loss_value = test_loss.to_scalar::<f32>()?;
    println!("测试损失: {:.6}", test_loss_value);
    
    // 断言测试损失足够小（拟合性能验证）
    assert!(test_loss_value < 0.2, "网络拟合性能不足，测试损失太大: {}", test_loss_value);
    
    Ok(())
}