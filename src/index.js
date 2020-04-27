import dotenv from 'dotenv';
import { join } from 'path'
import log from './log'
import DBL from 'dblapi.js'

import CompilerClient from './CompilerClient'
import SupportServer from './SupportServer'
import {StatisticsAPI} from './StatisticsTracking'

dotenv.config();

const client = new CompilerClient({
	prefix: process.env.BOT_PREFIX,
	loading_emote: process.env.LOADING_EMOTE,
	join_log: process.env.JOIN_LOG,
	dbl_log: process.env.DBL_LOG,
	compile_log: process.env.COMPILE_LOG,
	invite_link: process.env.INVITE_LINK,
	discordbots_link: process.env.DISCORDBOTS_LINK,
	github_link: process.env.GITHUB_LINK,
	stats_link: process.env.STATS_LINK,
	owner_id: process.env.OWNER_ID,
});

let maintenenceMdoe = true;

/**
 * API url in the form of https://url.com/
 */
let statisticsAPI = process.env.STATS_API_LINK;

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
 * @type {StatisticsAPI}
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

client.on('guildCreate', async (g) => {
	let guildCount = client.guilds.cache.size;
	if (shouldTrackStatistics)
		await statstracking.insertServerCount(guildCount);

	if (dblapi)
		await dblapi.postStats(guildCount);

	await supportserver.postJoined(g);

	if (maintenenceMdoe)
		await client.user.setPresence({activity: {name: `MAINTENENCE MODE`}, status: 'dnd'});
	else
		await client.user.setPresence({activity: {name: `in ${guildCount} servers | ;help`}, status: 'online'});

	log.info(`Client#guildCreate -> ${g.name}`);
})
.on('guildDelete', async (g) => {

	let guildCount = client.guilds.cache.size;
	if (shouldTrackStatistics)
		await statstracking.insertServerCount(guildCount);

	if (dblapi)
		await dblapi.postStats(guildCount);

	await supportserver.postLeft(g);

	if (maintenenceMdoe)
		await client.user.setPresence({activity: {name: `MAINTENENCE MODE`}, status: 'dnd'});
	else
		await client.user.setPresence({activity: {name: `in ${guildCount} servers | ;help`}, status: 'online'});


	log.info(`Client#guildDelete -> ${g.name}`);
})
.on('ready', async () => {
	log.info('Client#ready');
	client.hook();

	//Start up all internal tracking
	statstracking = new StatisticsAPI(client, statisticsAPI);
	supportserver = new SupportServer(client);
	
	client.setSupportServer(supportserver);
	await client.initialize();

	let guildCount = client.guilds.cache.size;

	if (maintenenceMdoe)
		await client.user.setPresence({activity: {name: `MAINTENENCE MODE`}, status: 'dnd'});
	else
		await client.user.setPresence({activity: {name: `in ${guildCount} servers | ;help`}, status: 'online'});


	if (shouldTrackStatistics)
		await statstracking.insertServerCount(guildCount);
	
	//Start dblapi tracking
	try {
		dblapi = setupDBL(client);
		if (dblapi)
			dblapi.postStats(guildCount);
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
	if (shouldTrackStatistics)
	{
		statstracking.commandExecuted(f.name);
		statstracking.incrementRequestCount();	
	}
	log.debug(`Client#commandExecuted -> ${f.name} command executed`);
})
.on('blacklistFailure', (error) => {
	log.error(`MessageRouter#Blacklist -> blacklist.json write failure (${error.message})`);
})
.on('compilersFailure', (error) => {
	log.error(`Compilers#compilersFailure -> ${error}`);
})
.on('commandExecutionError', (name, guild, error) => {
	log.error(`Client#commandExecutionError -> An error has occured in ${name} command: ${error} in ${guild.name}[${guild.id}]`)
});

client.login(process.env.BOT_TOKEN);