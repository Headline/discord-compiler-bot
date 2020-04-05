import dotenv from 'dotenv';
import CompilerClient from './CompilerClient'
import { join } from 'path'
import log from './log'
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
});

let supportserver = null;
let statstracking = null;

client.commands.registerCommandsIn(join(__dirname, 'commands'));

client.on('guildCreate', g => {
	if (statstracking)
		statstracking.inc();
	supportserver.postJoined(g);
	log.debug(`Client#guildCreate -> ${g.name}`);
})
.on('guildDelete', g => {
	if (statstracking)
		statstracking.dec();
	supportserver.postLeft(g);
	log.debug(`Client#guildDelete -> ${g.name}`);
})
.on('ready', async () => {
	log.info('Client#ready');
	client.hook();
	statstracking = new Servers(client.guilds.cache.size, client, process.env.DBL_TOKEN);
	supportserver = new SupportServer(client);
	
	client.setSupportServer(supportserver);
	await client.initializeCompilers();
	statstracking.updateAll();
})
.on('commandRegistered', (command) => {
	log.info(`Client#commandRegistered -> ${command.name}`);
})
.on('compilersReady', () => {
	log.info("Compilers#compilersReady");
})
.on('compilersFailure', (error) => {
	log.error(`Compilers#compilersFailure -> ${error}`);
})
.on('missingPermissions', (guild) => {
	log.warn(`Client#missingPermissions -> Missing permission in ${guild.name} [${guild.id}]`);
})
.on('commandExecuted', (f) => {
	Requests.doRequest();
	log.debug(`Client#commandExecuted -> ${f.name} command executed`);
})
client.login(process.env.BOT_TOKEN);
