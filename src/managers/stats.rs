pub struct StatsManager {
    servers: u64,
    shards: u32,
    boot_count: Vec<u64>,
    leave_queue: u64,
    join_queue: u64,
}

impl StatsManager {
    pub fn new() -> StatsManager {
        StatsManager {
            servers: 0,
            leave_queue: 0,
            join_queue: 0,
            shards: 0,
            boot_count: Vec::new(),
        }
    }

    /// Updates server count from boot
    pub fn post_servers(&mut self, amount: u64) {
        self.servers = amount;

        // in the connect phase it's entirely possible for our server count to be
        // zero while we receive a guild left or guild joined event, since they were
        // queued we can now modify the server count safely

        // join queue
        self.servers += self.join_queue;
        self.join_queue = 0;

        // leave queue
        self.servers -= self.leave_queue;
        self.leave_queue = 0;
    }

    /// Registers a new server
    pub fn new_server(&mut self) {
        if self.servers < 1 {
            // not all shards have loaded in yet - queue the join for post_servers
            self.join_queue += 1;
            return;
        }
        self.servers += 1;
    }

    /// Registers a server leave
    pub fn leave_server(&mut self) {
        if self.servers < 1 {
            // not loaded in - queue leave for post_servers
            self.leave_queue += 1;
            return;
        }
        self.servers -= 1;
    }

    pub fn server_count(&self) -> u64 {
        self.servers
    }

    pub fn shard_count(&self) -> u32 {
        self.shards
    }

    pub fn add_shard(&mut self, server_count: u64) {
        self.shards += 1;
        self.boot_count.push(server_count);
    }

    pub fn get_boot_vec_sum(&self) -> u64 {
        self.boot_count.iter().sum()
    }
}
