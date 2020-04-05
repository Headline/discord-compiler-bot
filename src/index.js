import dotenv from 'dotenv';
import CompilerClient from './CompilerClient'
import { join } from 'path'
import log from './log'

dotenv.config();

const client = new CompilerClient({
	prefix: process.env.BOT_PREFIX,
	loading_emote: process.env.LOADING_EMOTE,
});


client.commands.registerCommandsIn(join(__dirname, 'commands'));

client.on('guildCreate', g => {
	log.debug(`Client#guildCreate -> ${g.name}`);
})
.on('guildDelete', g => {
	log.debug(`Client#guildDelete -> ${g.name}`);
})
.on('ready', async () => {
	log.info('Client#ready');
	client.hook();
	await client.initializeCompilers();
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
	log.error(`Client#missingPermissions -> Missing critical permission in ${guild.name} [${guild.id}]`);
})
client.login(process.env.BOT_TOKEN);
