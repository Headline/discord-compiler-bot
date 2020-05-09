import { ShardingManager } from 'discord.js'
import log from './log'
import dotenv from 'dotenv';

dotenv.config();

const manager = new ShardingManager('./build/bot.js', { 
	token: process.env.BOT_TOKEN,
	execArgv: ['--async-stack-traces', '--async-stack-traces']
});


/**
 * Sets the bot's presence to 'MAINTENENCE MODE' to alert users of work being done. This also
 * disables statistics tracking
 * @type {string}
 */
let maintenanceMode = false;

/**
 * Boolean to determine if statistics should be tracked
 * @type {boolean}
 */
let shouldTrackStatistics = (maintenanceMode)?false:process.env.TRACK_STATISTICS;


/**
 * Counter to identify when server count should be reported
 * @type {number}
 */
let shardCount = 0;


manager.spawn();

manager.on('shardCreate', shard =>  {
	shard.on('message', (msg) => {
		if (msg === 'ready') {
			shardCount++;

			if(shardCount != -1 && shardCount == manager.totalShards) {

				/**
				 * Emitted once all shards have completed their ready event
				 * @event ShardingManager#allShardsReady
				 * @type {None}
				 */
				manager.emit('allShardsReady');
			}
		}
	})
	.on('error', (error) => log.error(`Shard#error -> ${error.message}`))
	.on('death', (child) => log.warn(`Shard#death -> [Shard ${shard.id}] Died `))
	.on('disconnect', () => log.warn(`Shard#disconnect -> [Shard ${shard.id}] Disconnected`))
	.on('reconnecting', () => log.warn(`Shard#reconnecting -> [Shard ${shard.id}] Attempting reconnection`));

	log.info(`ShardingManager#shardCreate -> [Shard ${shard.id}] Created successfully`)
})
.on('allShardsReady', async () => {
	shardCount = -1;
	const guildCounts = await manager.fetchClientValues('guilds.cache.size');
	const guildCount = guildCounts.reduce((a, b) => a + b, 0)

	manager.broadcastEval(`
		if (${maintenanceMode})
			this.user.setPresence({activity: {name: 'MAINTENENCE MODE'}, status: 'dnd'});
		else
			this.user.setPresence({activity: {name: 'in ${guildCount} servers | ;invite'}, status: 'online'});
		
		if (${shouldTrackStatistics})
			this.stats.insertServerCount(guildCount);	
	`)
});