import { Guild, MessageEmbed } from 'discord.js'

import fetch from 'node-fetch'
import log from './log'
import DBL from 'dblapi.js'

/**
 * A helper class which abstracts all support server information postings. 
 */
export default class SupportServer {

    /**
     * Posts a notification to the support guild when a user has voted
     * 
     * @param {DBL.User} user DBL User Info
     * @param {string} token bot token
     * @param {string} channel channel snowflake
     */
    static postVote(user, token, channel)
    {
        if (!channel)
            return;

        const embed = new MessageEmbed()
        .setDescription(`${user.username}#${user.discriminator} voted for us on top.gg!`)
        .setThumbnail('https://i.imgur.com/VXbdwSQ.png');

        SupportServer.manualDispatch(channel, token, embed, '');
    }

    static postAsm(code, lang, author, guild, success, failoutput, channel, token) {
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
        .setTitle('Assembly Requested:')    
        .setColor((success)?0x00FF00:0xFF0000)
        .addField("Language", lang, true)
        .addField("User",  author.tag, true)
        .addField("User ID",  author.id, true)
        .addField("Guild",  guild.name, true)
        .addField("Guild ID",  guild.id, true)
        .addField('Code', `\`\`\`${code}\n\`\`\`\n`);
        if (!success)
            embed.addField('Compiler Output', `\`\`\`${failoutput}\n\`\`\`\n`);
        
        SupportServer.manualDispatch(channel, token, embed, '');
    }

    /**
     * Posts to the join log of the support server when the bot enters a new guild
     * 
     * @param {Guild} guild
     * @param {string} token bot token
     * @param {string} channel channel snowflake
     */
    static postJoined(guild, token, channel)
    {
        if (!channel)
            return;

        const embed = new MessageEmbed()
        .setThumbnail(guild.iconURL)
        .setTitle('Server Joined:')    
        .setColor(0x00FF00)
        .addField("Name", guild.name, true)
        .addField("Guild Id",  guild.id, true)
        .addField("Total Members", guild.memberCount, true)
        .addField("Total Channels", guild.channels.cache.size, true)
        .addField("Guild Owner", guild.ownerID, true)
        .addField("Guild Region", guild.region, true)
        .addField("Creation Date", guild.createdAt.toISOString(), true)
        
        SupportServer.manualDispatch(channel, token, embed, '');
    }

    /**
     * Posts to the join log of the support server when the bot leaves a guild
     * 
     * @param {Guild} guild
     * @param {string} token bot token
     * @param {string} channel channel snowflake
     */
    static postLeft(guild, token, channel)
    {
        if (!channel)
            return;

        const embed = new MessageEmbed()
        .setThumbnail(guild.iconURL)
        .setTitle('Server Left:')    
        .setColor(0xFF0000)
        .addField("Name", guild.name, true)
        .addField("Guild Id",  guild.id, true)
        .addField("Total Members", guild.memberCount, true)
        .addField("Total Channels", guild.channels.cache.array.length, true)
        .addField("Guild Owner", guild.ownerID, true)
        .addField("Guild Region", guild.region, true)
        .addField("Creation Date", guild.createdAt.toISOString(), true)

        SupportServer.manualDispatch(channel, token, embed, '');
    }

    /**
     * Posts a compilation notice to the given channel
     * 
     * @param {string} code 
     * @param {string} lang 
     * @param {string} url 
     * @param {string} author 
     * @param {Guild} guild 
     * @param {boolean} success 
     * @param {string} failoutput 
     * @param {string} channel 
     * @param {string} token 
     */
    static postCompilation(code, lang, url, author, guild, success, failoutput, channel, token) {
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
        
        SupportServer.manualDispatch(channel, token, embed, '');
    }

    /**
     * Posts that a blacklisted user or guild has attempted to use the bot
     * 
     * @param {string} author 
     * @param {Guild} guild 
     * @param {string} channel channel snowflake
     * @param {boolean} isAuthorBanned 
     */
    static postBlacklistAttempt(author, guild, channel, token, isAuthorBanned) {
        if (!channel)
            return;

        const embed = new MessageEmbed()
        .setTitle('Blocked Request:')    
        .setColor(0xFF4500)
        .addField("User",  author.tag, true)
        .addField("User ID",  author.id, true)
        .addField("Guild",  guild.name, true)
        .addField("Guild ID",  guild.id, true)
        .addField("Ban Type", (isAuthorBanned)?"Author":"Server", true);
        
        SupportServer.manualDispatch(channel, token, embed, '');
    }

    /**
     * Manually sends a message skipping discord.js shit for sharding
     * 
     * @param {string} channel channel snowflake
     * @param {string} token bot authentication token
     * @param {MessageEmbed} embed embed to send
     * @param {string} content message to send
     */
    static async manualDispatch(channel, token, embed, content) {

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
