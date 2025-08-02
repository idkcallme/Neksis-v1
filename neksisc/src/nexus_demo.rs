// NEXUS CORE Demo - Real-Time Systems Programming with Neksis
//
// This demo showcases the NEXUS-RT real-time system capabilities,
// demonstrating how Neksis 2025 can handle time-critical embedded applications.

use neksisc::nexus_rt::*;
use std::time::Duration;

fn main() {
    println!("🌌 NEXUS CORE - Advanced Systems Programming Demo");
    println!("===================================================");
    println!("Built on Neksis 2025 - The Future of Systems Programming\n");

    demo_nexus_rt();
    demo_deterministic_memory();
    
    println!("\n🎉 NEXUS CORE Demo Complete!");
    println!("💡 This demonstrates Neksis's evolution from simple language to systems programming beast!");
    println!("🚀 Ready for embedded systems, robotics, and real-time applications!");
}

fn demo_nexus_rt() {
    println!("🚀 NEXUS-RT: Real-Time Scheduler Demo");
    println!("=====================================");
    
    let mut scheduler = RealTimeScheduler::new();
    
    // Create various real-time tasks
    
    // Critical sensor reading task (hard real-time)  
    let sensor_task = RealTimeTask::new_hard_rt(
        0,
        "Sensor Reader".to_string(),
        RealTimePriority::Critical,
        Duration::from_millis(10),  // 10ms deadline
        Duration::from_millis(20),  // 20ms period (50Hz)
        Duration::from_millis(2),   // 2ms execution time
    );
    
    // Motor control task (hard real-time)
    let motor_task = RealTimeTask::new_hard_rt(
        0,
        "Motor Controller".to_string(),
        RealTimePriority::High,
        Duration::from_millis(25),  // 25ms deadline
        Duration::from_millis(50),  // 50ms period (20Hz)
        Duration::from_millis(15),  // 15ms execution time
    );
    
    // Data logging task (soft real-time)
    let logging_task = RealTimeTask::new_soft_rt(
        0,
        "Data Logger".to_string(),
        RealTimePriority::Medium,
        Duration::from_millis(100), // 100ms deadline
        Duration::from_millis(200), // 200ms execution time
        Some(Duration::from_millis(500)), // 500ms period (2Hz)
    );
    
    // Background cleanup task (best effort)
    let cleanup_task = RealTimeTask::new_best_effort(
        0,
        "Memory Cleanup".to_string(),
        RealTimePriority::Low,
        Duration::from_millis(50), // 50ms execution time
    );
    
    // Add tasks to scheduler
    let sensor_id = scheduler.add_task(sensor_task);
    let motor_id = scheduler.add_task(motor_task);
    let logging_id = scheduler.add_task(logging_task);
    let cleanup_id = scheduler.add_task(cleanup_task);
    
    println!("📋 Registered {} real-time tasks:", 4);
    println!("  • Sensor Reader (Critical, 50Hz)");
    println!("  • Motor Controller (High, 20Hz)");
    println!("  • Data Logger (Medium, 2Hz)");
    println!("  • Memory Cleanup (Low, Best Effort)");
    
    // Start the scheduler
    scheduler.start();
    
    // Print task statistics
    println!("\n📊 Task Performance Statistics:");
    if let Some(stats) = scheduler.get_task_stats(sensor_id) {
        println!("  Sensor Reader: {} runs, {} missed deadlines", 
            stats.run_count, stats.missed_deadlines);
    }
    if let Some(stats) = scheduler.get_task_stats(motor_id) {
        println!("  Motor Controller: {} runs, {} missed deadlines", 
            stats.run_count, stats.missed_deadlines);
    }
    if let Some(stats) = scheduler.get_task_stats(logging_id) {
        println!("  Data Logger: {} runs, {} missed deadlines", 
            stats.run_count, stats.missed_deadlines);
    }
    if let Some(stats) = scheduler.get_task_stats(cleanup_id) {
        println!("  Memory Cleanup: {} runs, {} missed deadlines", 
            stats.run_count, stats.missed_deadlines);
    }
    
    scheduler.stop();
}

fn demo_deterministic_memory() {
    println!("\n🧠 NEXUS-RT: Deterministic Memory Demo");
    println!("=====================================");
    
    let mut memory = DeterministicMemory::new(4096); // 4KB pool
    
    println!("💾 Created deterministic memory pool: 4KB");
    
    // Simulate embedded system memory allocation patterns
    println!("\n🔄 Simulating embedded system memory usage:");
    
    // Allocate sensor buffer
    match memory.alloc(512) {
        Ok(_) => println!("✅ Allocated 512B sensor buffer"),
        Err(e) => println!("❌ Failed to allocate sensor buffer: {}", e),
    }
    
    // Allocate communication buffer
    match memory.alloc(1024) {
        Ok(_) => println!("✅ Allocated 1KB communication buffer"),
        Err(e) => println!("❌ Failed to allocate communication buffer: {}", e),
    }
    
    // Allocate processing buffer
    match memory.alloc(256) {
        Ok(_) => println!("✅ Allocated 256B processing buffer"),
        Err(e) => println!("❌ Failed to allocate processing buffer: {}", e),
    }
    
    // Show memory statistics
    let stats = memory.get_stats();
    println!("\n📊 Memory Pool Statistics:");
    println!("  Total Size: {}B", stats.pool_size);
    println!("  Allocated: {}B", stats.allocated);
    println!("  Free: {}B", stats.free);
    println!("  Utilization: {:.1}%", stats.utilization);
    
    // Test memory pressure
    println!("\n⚡ Testing memory pressure...");
    match memory.alloc(3000) {
        Ok(_) => println!("✅ Large allocation succeeded"),
        Err(e) => println!("⚠️  Expected failure: {}", e),
    }
    
    // Cleanup some memory
    memory.dealloc(512);
    println!("🧹 Freed 512B sensor buffer");
    
    let final_stats = memory.get_stats();
    println!("📊 Final utilization: {:.1}%", final_stats.utilization);
}

// Bonus: Demonstrate integration with Neksis language features
fn demo_neksis_integration() {
    println!("\n🔗 NEXUS-CORE + Neksis Language Integration");
    println!("==========================================");
    
    // Show how NEXUS-RT integrates with Neksis collections
    use neksisc::collections::*;
    
    let mut task_queue = NeksisVec::new();
    task_queue.push("Sensor Reading".to_string());
    task_queue.push("Motor Control".to_string());
    task_queue.push("Data Logging".to_string());
    
    println!("📋 Task queue with {} items:", task_queue.len());
    for (i, task) in task_queue.iter().enumerate() {
        println!("  {}. {}", i + 1, task);
    }
    
    // Show how NEXUS-RT can use Neksis OOP
    use neksisc::oop::*;
    use neksisc::modern_ast::Type;
    use std::collections::HashMap;
    
    println!("\n🏗️  Creating embedded system class hierarchy:");
    
    let mut registry = ClassRegistry::new();
    
    // Define embedded device base class
    let device_class = ClassDefinition {
        name: "EmbeddedDevice".to_string(),
        parent: None,
        fields: vec![
            FieldDefinition {
                name: "device_id".to_string(),
                field_type: Some(Type::Int),
                default_value: None,
                visibility: Visibility::Public,
            },
            FieldDefinition {
                name: "priority".to_string(),
                field_type: Some(Type::Int),
                default_value: None,
                visibility: Visibility::Public,
            },
        ],
        methods: vec![
            MethodDefinition {
                name: "initialize".to_string(),
                params: vec![],
                return_type: Some(Type::Boolean),
                body: vec![],
                visibility: Visibility::Public,
                is_static: false,
                is_virtual: true,
                is_override: false,
            },
        ],
        constructors: vec![],
        visibility: HashMap::new(),
    };
    
    registry.register_class(device_class).expect("Failed to register device class");
    println!("✅ Registered EmbeddedDevice base class");
    
    // Define sensor device (inherits from EmbeddedDevice)
    let sensor_class = ClassDefinition {
        name: "SensorDevice".to_string(),
        parent: Some("EmbeddedDevice".to_string()),
        fields: vec![
            FieldDefinition {
                name: "sensor_type".to_string(),
                field_type: Some(Type::String),
                default_value: None,
                visibility: Visibility::Public,
            },
            FieldDefinition {
                name: "sample_rate".to_string(),
                field_type: Some(Type::Int),
                default_value: None,
                visibility: Visibility::Public,
            },
        ],
        methods: vec![
            MethodDefinition {
                name: "read_sensor".to_string(),
                params: vec![],
                return_type: Some(Type::Float),
                body: vec![],
                visibility: Visibility::Public,
                is_static: false,
                is_virtual: false,
                is_override: false,
            },
        ],
        constructors: vec![],
        visibility: HashMap::new(),
    };
    
    registry.register_class(sensor_class).expect("Failed to register sensor class");
    println!("✅ Registered SensorDevice class (inherits from EmbeddedDevice)");
    
    // Test inheritance
    let inherits = registry.inherits_from("SensorDevice", "EmbeddedDevice");
    println!("🔍 SensorDevice inherits from EmbeddedDevice: {}", inherits);
}
