const os = require('os');
const botconfig = require('./../settings.json');
const Discord = require('discord.js');

module.exports.run = async (client, message, args, prefix) => {
    const memusage = process.memoryUsage().heapUsed /1024 / 1024; // memory in MB
    const cpuusage = os.loadavg()[0];
    const playercount = getUserCount(client);
    const guildcount = client.guilds.size;
    const invitelink = botconfig.invite_link;
    const votelink = botconfig.discordbots_link;

    const embed = new Discord.RichEmbed()
    .setTitle('Current Bot Info:')

    .setDescription("Discord Compiler Bot\n"
    + "Developed by Headline (Michael Flaherty)\n"
    + "==============================\n"
    + "[Invitation link]("+invitelink+")"
    + "\n[Vote for us!]("+votelink+")"
    + "\n[GitHub Repository](https://github.com/Headline/discord-compiler)"
    + "\n[Statistics Tracker](http://headlinedev.xyz/discord-compiler)"
    + "\n==============================\n")

    .setColor(0x00FF00)

    .addField("Total Users", formatNumber(playercount), true)
    .addField("Total Servers", formatNumber(guildcount), true)
    .addField("CPU Usage", formatNumber(cpuusage.toFixed(2)+"%"), true)
    .addField("Memory Usage", formatNumber(memusage.toFixed(2))+"MB", true)
    .addField("Average Ping", client.ping+"ms", true)
    .addField("Uptime", foramtTime(process.uptime()), true)
    .addField("System Info:", "**Node.js Version:** " + process.version
    + "\n**Operating System:** " + os.platform, false)

    .setFooter("Requested by: " + message.author.tag
    + " || powered by wandbox.org");

    message.channel.send(embed).catch(console.log);
}

/**
 * Time format
 * @param {Number} seconds
 */
function foramtTime(secs) {
    let seconds = Math.floor(secs);
    let hours = Math.floor(seconds / 3600) % 24;
    let minutes = Math.floor(seconds / 60) % 60;
    let seconds2 = seconds % 60;
    return [hours, minutes, seconds2]
        .map(v => v < 10 ? "0" + v : v)
        .filter((v,i) => v !== "00" || i > 0)
        .join(":");
}

/**
 * Formats a number in a readable fashion
 * @param {Number} num;
 */
function formatNumber(num) {
    return num.toString().replace(/(\d)(?=(\d{3})+(?!\d))/g, '$1,');
}

/**
 * Gets the amount of total users connected to all guilds.
 * 
 * @param {Discord.Client} client 
 */
function getUserCount(client) {
    let members = 0;
    client.guilds.forEach(guild => {
        members += guild.members.size;
    });
    return members;
}

module.exports.help = {
    name:"botinfo",
    description:"shows all of the information regarding the bot's state"
}