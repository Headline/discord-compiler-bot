import { Client, Guild, MessageEmbed, Channel } from 'discord.js'
import CompilerClient from './CompilerClient'
import log from './log'

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
     * @param {string} userid discord user id
     */
    async postVote(userid)
    {
        try {
            if (!this.client.dbl_log)
                return;

            /**
             * @type {Channel}
             */
            let channel = await this.client.channels.fetch(this.client.dbl_log);
            if (!channel)
                return;
            
            let user = await this.client.users.fetch(userid);
            await channel.send(`${user.tag} has just voted for us on top.gg!  :heart:`);
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

            /**
             * @type {Channel}
             */
            let channel = await this.client.channels.fetch(this.client.join_log);
            if (!channel)
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
            
            await channel.send(embed)
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

            /**
             * @type {Channel}
             */
            let channel = await this.client.channels.fetch(this.client.join_log);
            if (!channel)
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

            await channel.send(embed)
        }
        catch (err) {
            log.error(`SupportServer#postLeft -> ${err}`);
        }
    }

    async postCompilation(code, lang, url, author, guild, success, failoutput) {
        try {

            if (!this.client.compile_log)
                return;

            /**
             * @type {Channel}
             */
            let channel = await this.client.channels.fetch(this.client.compile_log);
            if (!channel)
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
            
            await channel.send(embed)
        }
        catch (err) {
            log.error(`SupportServer#postCompilation -> ${err}`);
        }
    }
}
