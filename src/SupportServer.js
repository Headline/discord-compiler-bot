import { Client, Guild, MessageEmbed, Channel, Constants } from 'discord.js'
import CompilerClient from './CompilerClient'

import fetch from 'node-fetch'
import log from './log'
import DBL from 'dblapi.js'
/**
 * A helper class which abstracts all support server information postings. 
 */
export default class SupportServer {
    /**
     * Creates a SupportServer object
     * 
     * @param {Client} client 
     */
    constructor(client) {
        /**
         * Discord client
         * 
         * @type {CompilerClient}
         */
        this.client = client;
    }

    /**
     * Posts a notification to the support guild when a user has voted
     * 
     * @param {DBL.User} user DBL User Info
     */
    async postVote(user)
    {
        try {
            if (!this.client.dbl_log)
                return;

            const embed = new MessageEmbed()
            .setDescription(`${user.username} voted for us on top.gg!  :heart:`);
            if (user.avatar)
                embed.setThumbnail(user.avatar)

            this.manualDispatch(this.client.dbl_log, this.client.token, embed, '');
        }
        catch (err) {
            log.error(`SupportServer#postVote -> ${err}`);
        }
    }

    /**
     * Posts to the join log of the support server when the bot enters a new guild
     * 
     * @param {Guild} guild
     */
    async postJoined(guild)
    {
        try {
            if (!this.client.join_log)
                return;

            guild = await guild.fetch();

            const embed = new MessageEmbed()
            .setThumbnail(guild.iconURL)
            .setTitle('Server Joined:')    
            .setColor(0x00FF00)
            .addField("Name", guild.name, true)
            .addField("Guild Id",  guild.id, true)
            .addField("Total Members", guild.memberCount, true)
            .addField("Total Channels", guild.channels.cache.size, true)
            .addField("Guild Owner", guild.owner.user.tag, true)
            .addField("Guild Region", guild.region, true)
            .addField("Creation Date", guild.createdAt.toISOString(), true)
            
            this.manualDispatch(this.client.join_log, this.client.token, embed, '');
        }
        catch (err) {
            log.error(`SupportServer#postJoined -> ${err}`);
        }
    }

    /**
     * Posts to the join log of the support server when the bot leaves a guild
     * 
     * @param {Guild} guild
     */
    async postLeft(guild)
    {
        try {
            if (!this.client.join_log)
                return;


            guild = await guild.fetch();

            const embed = new MessageEmbed()
            .setThumbnail(guild.iconURL)
            .setTitle('Server Left:')    
            .setColor(0xFF0000)
            .addField("Name", guild.name, true)
            .addField("Guild Id",  guild.id, true)
            .addField("Total Members", guild.memberCount, true)
            .addField("Total Channels", guild.channels.cache.array.length, true)
            .addField("Guild Owner", guild.owner.user.tag, true)
            .addField("Guild Region", guild.region, true)
            .addField("Creation Date", guild.createdAt.toISOString(), true)

            this.manualDispatch(this.client.join_log, this.client.token, embed, '');

        }
        catch (err) {
            log.error(`SupportServer#postLeft -> ${err}`);
        }
    }

    async postCompilation(code, lang, url, author, guild, success, failoutput) {
        try {

            if (!this.client.compile_log)
                return;
    
            if (code.length >= 1017) {
                code = code.substring(0, 1016);
            }
            if (failoutput) {
                if (failoutput.length > 1017) {
                    failoutput = failoutput.substring(0, 1016);
                }
            }

            const embed = new MessageEmbed()
            .setTitle('Compilation Requested:')    
            .setColor((success)?0x00FF00:0xFF0000)
            .addField("Language", lang, true)
            .addField("URL",  url, true)
            .addField("User",  author.tag, true)
            .addField("User ID",  author.id, true)
            .addField("Guild",  guild.name, true)
            .addField("Guild ID",  guild.id, true)
            .addField('Code', `\`\`\`${code}\n\`\`\`\n`);
            if (!success)
                embed.addField('Compiler Output', `\`\`\`${failoutput}\n\`\`\`\n`);
            
            this.manualDispatch(this.client.compile_log, this.client.token, embed, '');
        }
        catch (err) {
            log.error(`SupportServer#postCompilation -> ${err}`);
        }
    }

    /**
     * Manually sends a message skipping discord.js shit for sharding
     * 
     * @param {string} channel channel snowflake
     * @param {string} token bot authentication token
     * @param {MessageEmbed} embed embed to send
     * @param {string} content message to send
     */
    async manualDispatch(channel, token, embed, content) {

        /**
         * Allow me to write of my pain for a brief moment. 
         * 
         * This looks like a simple method, in fact it is. I cannot overlook the hours I've spent scouring
         * the discord.js codebase trying to send a message manually using their abstractions, while skipping
         * all of the caching they do. It's not as easy as it used to be. The amount of unnecessary 
         * abstraction and spaghetti code pathing has given me the realization that discord.js isn't
         * the beautiful library I once thought it as. Perhaps moving to rust is the answer. Anyway, I'd
         * like to thank node-fetch for being there whenever I need it
         */
        try {
            await fetch(`https://discordapp.com/api/v6/channels/${channel}/messages`, {
                method: "POST",
                body: JSON.stringify({
                    embed: embed.toJSON(),
                    tts: false,
                    content: content
                }),
                headers: {
                    'Authorization': `Bot ${token}`,
                    'Content-Type': 'application/json'
                },
            });
        }
        catch (err) {
            log.error(`SupportServer#manualDispatch -> ${err.message}`);
        }
    }
}
