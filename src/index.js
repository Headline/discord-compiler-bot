import { ShardingManager } from 'discord.js'
import log from './log'
import dotenv from 'dotenv';
import SupportServer from './SupportServer'

dotenv.config();

const manager = new ShardingManager('./build/bot.js', { 
	token: process.env.BOT_TOKEN,
	execArgv: ['--async-stack-traces', '--async-stack-traces']
});

/**
 * Counter to identify when server count should be reported
 * @type {number}
 */
let shardCount = 0;

manager.on('shardCreate', shard =>  {
	shard.on('message', (msg) => {
		if (msg === 'initialized') {
			if(++shardCount == manager.totalShards) {
				/**
				 * Emitted once all shards have completed their ready event
				 * @event ShardingManager#allShardsReady
				 * @type {None}
				 */
				manager.emit('shardsInitialized');
			}
		}
	})
	.on('error', (error) => log.error(`Shard#error -> ${error.message}`))
	.on('death', () => log.warn(`Shard#death -> [Shard ${shard.id}] Died `))
	.on('disconnect', () => log.warn(`Shard#disconnect -> [Shard ${shard.id}] Disconnected`))
	.on('reconnecting', () => log.warn(`Shard#reconnecting -> [Shard ${shard.id}] Attempting reconnection`));

	log.info(`ShardingManager#shardCreate -> [Shard ${shard.id}] Created successfully`)
})
.once('shardsInitialized', async () => {
	manager.broadcastEval(`
	(async () => {
		this.updatePresence();
		let count = await this.getTrueServerCount();
		this.updateServerCount(count);
	})();
	`)
});

manager.spawn();