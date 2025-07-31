use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub allocated_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub deallocated_bytes: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub current_allocations: usize,
    pub memory_fragmentation: f64,
}

#[derive(Debug, Clone)]
pub struct AllocationRecord {
    pub address: usize,
    pub size: usize,
    pub timestamp: Instant,
    pub stack_trace: Vec<String>,
    pub allocation_site: String,
}

#[derive(Debug, Clone)]
pub struct MemoryProfile {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub peak_memory_usage: usize,
    pub average_allocation_size: f64,
    pub memory_leaks: Vec<AllocationRecord>,
    pub allocation_patterns: HashMap<String, usize>,
    pub duration: Duration,
}

pub struct MemoryProfiler {
    stats: Arc<Mutex<MemoryStats>>,
    allocations: Arc<Mutex<HashMap<usize, AllocationRecord>>>,
    start_time: Instant,
    enabled: bool,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(MemoryStats {
                allocated_bytes: 0,
                peak_allocated_bytes: 0,
                deallocated_bytes: 0,
                allocation_count: 0,
                deallocation_count: 0,
                current_allocations: 0,
                memory_fragmentation: 0.0,
            })),
            allocations: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
            enabled: true,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn record_allocation(&self, address: usize, size: usize, allocation_site: &str) {
        if !self.enabled {
            return;
        }

        let record = AllocationRecord {
            address,
            size,
            timestamp: Instant::now(),
            stack_trace: self.capture_stack_trace(),
            allocation_site: allocation_site.to_string(),
        };

        if let Ok(mut stats) = self.stats.lock() {
            stats.allocated_bytes += size;
            stats.allocation_count += 1;
            stats.current_allocations += 1;
            
            if stats.allocated_bytes > stats.peak_allocated_bytes {
                stats.peak_allocated_bytes = stats.allocated_bytes;
            }
        }

        if let Ok(mut allocations) = self.allocations.lock() {
            allocations.insert(address, record);
        }
    }

    pub fn record_deallocation(&self, address: usize) {
        if !self.enabled {
            return;
        }

        if let Ok(mut stats) = self.stats.lock() {
            if let Ok(mut allocations) = self.allocations.lock() {
                if let Some(record) = allocations.remove(&address) {
                    stats.deallocated_bytes += record.size;
                    stats.deallocation_count += 1;
                    stats.current_allocations -= 1;
                }
            }
        }
    }

    pub fn get_stats(&self) -> MemoryStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            MemoryStats {
                allocated_bytes: 0,
                peak_allocated_bytes: 0,
                deallocated_bytes: 0,
                allocation_count: 0,
                deallocation_count: 0,
                current_allocations: 0,
                memory_fragmentation: 0.0,
            }
        }
    }

    pub fn generate_profile(&self) -> MemoryProfile {
        let stats = self.get_stats();
        let duration = self.start_time.elapsed();
        
        let mut allocation_patterns = HashMap::new();
        let mut memory_leaks = Vec::new();
        
        if let Ok(allocations) = self.allocations.lock() {
            for record in allocations.values() {
                *allocation_patterns.entry(record.allocation_site.clone()).or_insert(0) += 1;
                memory_leaks.push(record.clone());
            }
        }

        let average_allocation_size = if stats.allocation_count > 0 {
            stats.allocated_bytes as f64 / stats.allocation_count as f64
        } else {
            0.0
        };

        MemoryProfile {
            total_allocations: stats.allocation_count,
            total_deallocations: stats.deallocation_count,
            peak_memory_usage: stats.peak_allocated_bytes,
            average_allocation_size,
            memory_leaks,
            allocation_patterns,
            duration,
        }
    }

    pub fn detect_memory_leaks(&self) -> Vec<AllocationRecord> {
        if let Ok(allocations) = self.allocations.lock() {
            allocations.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_allocation_patterns(&self) -> HashMap<String, usize> {
        let mut patterns = HashMap::new();
        
        if let Ok(allocations) = self.allocations.lock() {
            for record in allocations.values() {
                *patterns.entry(record.allocation_site.clone()).or_insert(0) += 1;
            }
        }
        
        patterns
    }

    pub fn calculate_memory_fragmentation(&self) -> f64 {
        let stats = self.get_stats();
        
        if stats.allocated_bytes == 0 {
            return 0.0;
        }

        let fragmentation = if let Ok(allocations) = self.allocations.lock() {
            let mut total_gaps = 0;
            let mut sorted_addresses: Vec<usize> = allocations.keys().cloned().collect();
            sorted_addresses.sort();

            for i in 1..sorted_addresses.len() {
                let prev_record = allocations.get(&sorted_addresses[i - 1]).unwrap();
                let curr_record = allocations.get(&sorted_addresses[i]).unwrap();
                
                let gap = curr_record.address - (prev_record.address + prev_record.size);
                total_gaps += gap;
            }

            total_gaps as f64 / stats.allocated_bytes as f64
        } else {
            0.0
        };

        fragmentation
    }

    fn capture_stack_trace(&self) -> Vec<String> {
        // This is a simplified stack trace capture
        // In a real implementation, you would use a proper stack trace library
        vec![
            "stack_trace::capture".to_string(),
            "memory_profiler::record_allocation".to_string(),
        ]
    }

    pub fn print_summary(&self) {
        let profile = self.generate_profile();
        let stats = self.get_stats();
        
        println!("=== Memory Profile Summary ===");
        println!("Duration: {:?}", profile.duration);
        println!("Total allocations: {}", profile.total_allocations);
        println!("Total deallocations: {}", profile.total_deallocations);
        println!("Peak memory usage: {} bytes", profile.peak_memory_usage);
        println!("Average allocation size: {:.2} bytes", profile.average_allocation_size);
        println!("Current allocations: {}", stats.current_allocations);
        println!("Memory fragmentation: {:.2}%", self.calculate_memory_fragmentation() * 100.0);
        
        if !profile.memory_leaks.is_empty() {
            println!("⚠️  Potential memory leaks detected: {}", profile.memory_leaks.len());
            for leak in &profile.memory_leaks[..std::cmp::min(5, profile.memory_leaks.len())] {
                println!("  - {} bytes at 0x{:x} ({})", 
                        leak.size, leak.address, leak.allocation_site);
            }
        }
        
        println!("Top allocation sites:");
        let mut sorted_patterns: Vec<_> = profile.allocation_patterns.iter().collect();
        sorted_patterns.sort_by(|a, b| b.1.cmp(a.1));
        
        for (site, count) in sorted_patterns.iter().take(5) {
            println!("  - {}: {} allocations", site, count);
        }
    }

    pub fn start_periodic_reporting(&self, interval: Duration) {
        let stats = Arc::clone(&self.stats);
        let allocations = Arc::clone(&self.allocations);
        let start_time = self.start_time;
        
        thread::spawn(move || {
            loop {
                thread::sleep(interval);
                
                if let Ok(stats) = stats.lock() {
                    println!("Memory Stats: {} bytes allocated, {} current allocations", 
                            stats.allocated_bytes, stats.current_allocations);
                }
            }
        });
    }
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

// Memory allocation hooks for integration with the VM
pub struct MemoryHooks {
    profiler: Arc<MemoryProfiler>,
}

impl MemoryHooks {
    pub fn new(profiler: Arc<MemoryProfiler>) -> Self {
        Self { profiler }
    }

    pub fn on_allocation(&self, address: usize, size: usize, site: &str) {
        self.profiler.record_allocation(address, size, site);
    }

    pub fn on_deallocation(&self, address: usize) {
        self.profiler.record_deallocation(address);
    }
}

// Memory optimization suggestions
pub struct MemoryOptimizer {
    profiler: Arc<MemoryProfiler>,
}

impl MemoryOptimizer {
    pub fn new(profiler: Arc<MemoryProfiler>) -> Self {
        Self { profiler }
    }

    pub fn generate_optimization_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        let profile = self.profiler.generate_profile();
        
        // Check for memory leaks
        if !profile.memory_leaks.is_empty() {
            suggestions.push(format!(
                "Found {} potential memory leaks. Consider adding explicit deallocations.",
                profile.memory_leaks.len()
            ));
        }
        
        // Check allocation patterns
        for (site, count) in &profile.allocation_patterns {
            if *count > 1000 {
                suggestions.push(format!(
                    "High allocation frequency at '{}' ({} allocations). Consider object pooling.",
                    site, count
                ));
            }
        }
        
        // Check average allocation size
        if profile.average_allocation_size < 64.0 {
            suggestions.push(
                "Many small allocations detected. Consider using memory pools for small objects.".to_string()
            );
        }
        
        // Check fragmentation
        let fragmentation = self.profiler.calculate_memory_fragmentation();
        if fragmentation > 0.3 {
            suggestions.push(
                "High memory fragmentation detected. Consider defragmentation or different allocation strategy.".to_string()
            );
        }
        
        suggestions
    }
} 