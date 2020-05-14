import { ShardingManager } from 'discord.js'
import dotenv from 'dotenv';
import DBL from 'dblapi.js'

import log from './log'
import SupportServer from './SupportServer'

dotenv.config();

const manager = new ShardingManager('./build/bot.js', { 
	token: process.env.BOT_TOKEN,
	execArgv: ['--async-stack-traces']
});

/**
 * DBL Api object
 * @type {DBL}
 */
let dbl = null;

/**
 * Sets up Discord Bot List for public server count display,
 * bot info, and webhooks. Should be called after Discord.Client ready events
 */
function setupDBL() {
	if (!process.env.DBL_TOKEN) {
		return null;
	}

	let options = {};

	// If we have webhook capability, lets set it up
	if (process.env.DBL_WEBHOOK_PORT && process.env.DBL_WEBHOOK_PASSWORD) {
			options.webhookPort = process.env.DBL_WEBHOOK_PORT;
			options.webhookAuth = process.env.DBL_WEBHOOK_PASSWORD;
	}

	let dblapi = new DBL(process.env.DBL_TOKEN, options);
	if (dblapi.webhook) {
		dblapi.webhook.on('ready', (hook) => {
			log.info(`DBL#ready -> Webhook running at http://${hook.hostname}:${hook.port}${hook.path}`)
		})
		.on('vote', async (vote) => {
			let u = await dblapi.getUser(vote.user);
			SupportServer.postVote(u, process.env.BOT_TOKEN, process.env.DBL_LOG);
		});
	}

	dblapi.on('posted', () => {
		log.info('DBL#posted -> Server count posted');
	})
	.on('error', (e) => {
		log.warn(`DBL#error -> DBL failure: ${e}`);
	});

	return dblapi;
}

/**
 * Counter to identify when server count should be reported
 * @type {number}
 */
let shardCount = 0;

manager.on('shardCreate', shard =>  {
	shard.on('message', async (msg) => {
		switch (msg) {
			case 'initialized':
				if(++shardCount == manager.totalShards) {
					/**
					 * Emitted once all shards have completed their ready event
					 * @event ShardingManager#allShardsReady
					 * @type {None}
					 */
					manager.emit('shardsInitialized');
				}
			case 'updateDBL':
				if (dbl) {
					let values = await manager.fetchClientValues('guilds.cache.size');
					dbl.postStats(values);
				}
				break;
		}
	})
	.on('error', (error) => log.error(`Shard#error -> ${error.message}`))
	.on('death', () => log.warn(`Shard#death -> [Shard ${shard.id}] Died `))
	.on('disconnect', () => log.warn(`Shard#disconnect -> [Shard ${shard.id}] Disconnected`))
	.on('reconnecting', () => log.warn(`Shard#reconnecting -> [Shard ${shard.id}] Attempting reconnection`));

	log.info(`ShardingManager#shardCreate -> [Shard ${shard.id}] Created successfully`)
})
.once('shardsInitialized', async () => {
	dbl = setupDBL();
	manager.broadcastEval(`
	(async () => {
		this.updatePresence();
		let count = await this.getTrueServerCount();
		this.updateServerCount(count);
	})();
	`)
});

manager.spawn();