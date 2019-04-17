const Discord = require('discord.js');
const botconfig = require('./settings.json');

/**
 * A helper class which abstracts all support server information postings. 
 */
class SupportServer {
    /**
     * Creates a SupportServer object & sets necessary instance variables for proper function
     * 
     * @param {Discord.Client} client 
     */
    constructor(client) {
        let guild = null;
         client.guilds.forEach((g) => {
            if (g.id == botconfig.support_server) {
                guild = g;
            }
        })
        
        this.supportguild = guild;
    }

    /**
     * Posts to the join log of the support server for tracking.
     * 
     * @param {Discord.Guild} guild
     */
    postJoined(guild)
    {
        if (this.supportguild == null)
            return;

        let channel = null;
        this.supportguild.channels.forEach((element) => {
            if (element.name === "join-log")
                channel = element;
        });

        if (channel == null)
            return;

        const embed = new Discord.RichEmbed()
        .setThumbnail(guild.iconURL)
        .setTitle('Server Joined:')    
        .setColor(0x00FF00)
        .addField("Name", guild.name, true)
        .addField("Guild Id",  guild.id, true)
        .addField("Total Members", guild.memberCount, true)
        .addField("Total Channels", guild.channels.array.length, true)
        .addField("Guild Owner", guild.owner.user.tag, true)
        .addField("Guild Region", guild.region, true)
        .addField("Creation Date", guild.createdAt.toISOString(), true)
        channel.send(embed).catch();
    }

    /**
     * Posts to the join log of the support server for tracking.
     * 
     * @param {Discord.Guild} guild
     */
    postLeft(guild)
    {
        if (this.supportguild == null)
            return;

        let channel = null;
        
        this.supportguild.channels.forEach((element) => {
            if (element.name === "join-log")
                channel = element;
        });

        if (channel == null)
            return;

        const embed = new Discord.RichEmbed()
        .setThumbnail(guild.iconURL)
        .setTitle('Server Left:')    
        .setColor(0xFF0000)
        .addField("Name", guild.name, true)
        .addField("Guild Id",  guild.id, true)
        .addField("Total Members", guild.memberCount, true)
        .addField("Total Channels", guild.channels.array.length, true)
        .addField("Guild Owner", guild.owner.user.tag, true)
        .addField("Guild Region", guild.region, true)
        .addField("Creation Date", guild.createdAt.toISOString(), true)
        channel.send(embed).catch(console.log);
    }

    postCompilation(code, lang, url, author, guild) {
        if (this.supportguild == null)
            return;

        let channel = null;

        this.supportguild.channels.forEach((element) => {
        if (element.name === "compile-log")
            channel = element;
        });

        if (channel == null)
            return;

        const embed = new Discord.RichEmbed()
        .setTitle('Compilation Requested:')    
        .setColor(0x00FF00)
        .addField("Language", lang, true)
        .addField("URL",  url, true)
        .addField("User",  author.tag, true)
        .addField("Guild",  guild.name, true)
        .addField('Code', `\`\`\`${code}\n\`\`\`\n`);
        channel.send(embed).catch(console.log);
    }
}

module.exports = {SupportServer};