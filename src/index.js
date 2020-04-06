import dotenv from 'dotenv';
import { join } from 'path'
import log from './log'
import DBL from 'dblapi.js'

import CompilerClient from './CompilerClient'
import SupportServer from './SupportServer'
import {Servers, Requests} from './StatisticsTracking'

dotenv.config();

const client = new CompilerClient({
	prefix: process.env.BOT_PREFIX,
	loading_emote: process.env.LOADING_EMOTE,
	support_server: process.env.SUPPORT_SERVER,
	invite_link: process.env.INVITE_LINK,
	discordbots_link: process.env.DISCORDBOTS_LINK,
	github_link: process.env.GITHUB_LINK,
	stats_link: process.env.STATS_LINK,
	owner_id: process.env.OWNER_ID,
});

/**
 * Boolean to determine if statistics should be tracked
 * @type {boolean}
 */
let shouldTrackStatistics = process.env.TRACK_STATISTICS;

/**
 * Support server communication link
 * @type {SupportServer}
 */
let supportserver = null;

/**
 * Statistics tracking helper class
 * @type {Servers}
 */
let statstracking = null;

/**
 * DBL Api object
 * @type {DBL}
 */
let dblapi = null;

/**
 * Sets up Discord Bot List for public server count display,
 * bot info, and webhooks. Should be called after Discord.Client ready event
 * 
 * @param {CompilerClient} client ready client instance
 */
function setupDBL(client) {
	if (!process.env.DBL_TOKEN) {
		return null;
	}

	// If we have webhook capability
	if (process.env.DBL_WEBHOOK_PORT && process.env.DBL_WEBHOOK_PASSWORD) {
		let options = {
			webhookPort: process.env.DBL_WEBHOOK_PORT, 
			webhookAuth: process.env.DBL_WEBHOOK_PASSWORD,
		};

		dblapi = new DBL(process.env.DBL_TOKEN, options, client);
		dblapi.webhook.on('ready', (hook) => {
			log.info(`DBL#ready -> Webhook running at http://${hook.hostname}:${hook.port}${hook.path}`)
		})
		.on('vote', async (vote) => {
			await supportserver.postVote(vote.user);
		});
		
	}
	// No webhooks available, lets just set up default stuff
	else {
		dblapi = new DBL(process.env.DBL_TOKEN, client);
	}

	dblapi.on('posted', () => {
		log.info('DBL#posted -> Server count posted');
	})
	.on('error', (e) => {
		log.warn(`DBL#error -> DBL failure: ${e}`);
	});

	return dblapi;
}

client.commands.registerCommandsIn(join(__dirname, 'commands'));

client.on('guildCreate', g => {
	if (shouldTrackStatistics)
		statstracking.inc();
	supportserver.postJoined(g);
	log.debug(`Client#guildCreate -> ${g.name}`);
})
.on('guildDelete', g => {
	if (shouldTrackStatistics)
		statstracking.dec();
	supportserver.postLeft(g);
	log.debug(`Client#guildDelete -> ${g.name}`);
})
.on('ready', async () => {
	log.info('Client#ready');
	client.hook();
	statstracking = new Servers(client.guilds.cache.size, client);
	supportserver = new SupportServer(client);
	
	client.setSupportServer(supportserver);
	await client.initialize();
	if (shouldTrackStatistics)
		statstracking.updateAll();
	
	try {
		dblapi = setupDBL(client);
	}
	catch (error)
	{
		log.error(`DBL$dblSetup -> ${error}`);
	}
})
.on('commandRegistered', (command) => {
	log.info(`Client#commandRegistered -> ${command.name}`);
})
.on('compilersReady', () => {
	log.info("Compilers#compilersReady");
})
.on('missingPermissions', (guild) => {
	log.warn(`Client#missingPermissions -> Missing permission in ${guild.name} [${guild.id}]`);
})
.on('commandExecuted', (f) => {
	Requests.doRequest();
	log.debug(`Client#commandExecuted -> ${f.name} command executed`);
})
.on('blacklistFailure', (error) => {
	log.error(`MessageRouter#Blacklist -> blacklist.json write failure (${error.message})`);
})
.on('compilersFailure', (error) => {
	log.error(`Compilers#compilersFailure -> ${error}`);
})

client.login(process.env.BOT_TOKEN);