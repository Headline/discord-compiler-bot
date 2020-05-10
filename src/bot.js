import dotenv from 'dotenv';
import { join } from 'path'
import log from './log'
import DBL from 'dblapi.js'

import CompilerClient from './CompilerClient'

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
	stats_api_link: process.env.STATS_API_LINK,

	ws: {
		intents: ["GUILDS", "GUILD_MESSAGES", "GUILD_MESSAGE_REACTIONS"]
	}
});

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
		.on('vote', async (bot, user) => {
			let u = await dblapi.getUser(user);
			client.supportServer.postVote(u);
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
	const count = await client.getTrueServerCount();

	client.updateServerCount(count);

	if (dblapi)
		dblapi.postStats(count);

	client.supportServer.postJoined(g);

	client.updatePresence();

	log.info(`Client#guildCreate -> ${g.name}`);
})
.on('guildDelete', async (g) => {
	const count = await client.getTrueServerCount();
	client.updateServerCount(count);

	if (dblapi)
		dblapi.postStats(count);

	client.supportServer.postLeft(g);

	client.updatePresence();

	log.info(`Client#guildDelete -> ${g.name}`);
})
.on('ready', async () => {
	log.info('Client#ready');
	client.hook();

	client.initialize();	

	//Start dblapi tracking
	try {
		dblapi = setupDBL(client);
		if (dblapi) {
			let count = await client.getTrueServerCount();
			dblapi.postStats(count);
		}
	}
	catch (error)
	{
		log.error(`DBL#dblSetup -> ${error}`);
	}

	/**
	 * Tell shard manager that we're good to go.
	 */
	client.shard.send('initialized');
})
.on('commandRegistered', (command) => {
	log.debug(`Client#commandRegistered -> ${command.name}`);
})
.on('compilersReady', () => {
	log.info("Compilers#compilersReady");
})
.on('missingPermissions', (guild) => {
	log.warn(`Client#missingPermissions -> Missing permission in ${guild.name} [${guild.id}]`);
})
.on('commandExecuted', (f) => {
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