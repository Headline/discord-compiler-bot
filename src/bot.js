import dotenv from 'dotenv';
import { join } from 'path'
import log from './log'

import CompilerClient from './CompilerClient'
import SupportServer from './SupportServer'

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

client.commands.registerCommandsIn(join(__dirname, 'commands'));

client.on('guildCreate', async (g) => {
	const count = await client.getTrueServerCount();
	client.updateServerCount(count);

	SupportServer.postJoined(g, client.token, client.join_log);

	client.updatePresence();

	log.info(`Client#guildCreate -> ${g.name}`);
	client.shard.send('updateDBL');
})
.on('guildDelete', async (g) => {
	const count = await client.getTrueServerCount();
	client.updateServerCount(count);

	SupportServer.postLeft(g, client.token, client.join_log);

	client.updatePresence();

	log.info(`Client#guildDelete -> ${g.name}`);
	client.shard.send('updateDBL');
})
.on('ready', async () => {
	client.hook();

	client.initialize();	
	/**
	 * Tell shard manager that we're good to go.
	 */
	client.shard.send('initialized');
	log.info('Client#ready');
})
.on('commandRegistered', (command) => {
	log.debug(`Client#commandRegistered -> ${command.name}`);
})
.on('wandboxReady', () => {
	log.info("Wandbox#wandboxReady");
})
.on('godboltReady', () => {
	log.info("Godbolt#godboltReady");
})
.on('missingPermissions', (guild, err) => {
	log.warn(`Client#missingPermissions -> Missing permission in ${guild.name} [${guild.id}]: ${err}`);
})
.on('commandExecuted', (f) => {
	log.debug(`Client#commandExecuted -> ${f.name} command executed`);
})
.on('blacklistFailure', (error) => {
	log.error(`MessageRouter#Blacklist -> blacklist.json write failure (${error.message})`);
})
.on('wandboxFailure', (error) => {
	log.error(`Compilers#wandboxFailure -> ${error}`);
})
.on('godboltFailure', (error) => {
	log.error(`Client#godboltFailure -> ${error}`);
})
.on('commandExecutionError', (name, guild, error) => {
	log.error(`Client#commandExecutionError -> An error has occured in ${name} command: ${error} in ${guild.name}[${guild.id}]`)
});

client.login(process.env.BOT_TOKEN);