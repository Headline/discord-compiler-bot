import os from 'os'
import { Message, MessageEmbed, Client} from 'discord.js'
import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'

export default class BotInfoCommand extends CompilerCommand {
    /**
     *  Creates the Compile command
     */
    constructor(client) {
        super(client, {
            name: 'botinfo',
            description: 'Displays the bot\'s state information',
            developerOnly: false
        });
    }

    /**
     * Function which is executed when the command is requested by a user
     *
     * @param {CompilerCommandMessage} msg
     */
    async run(msg) {
        const memusage = process.memoryUsage().heapUsed / 1024 / 1024; // memory in MB
        const cpuusage = os.loadavg()[0];
        const playercount = this.getUserCount(this.client);
        const guildcount = this.client.guilds.cache.size;
        const invitelink = this.client.invite_link;
        const votelink = this.client.discordbots_link;
		const githublink = this.client.github_link;
		const statslink = this.client.stats_link;
		
        const embed = new MessageEmbed()
            .setTitle('Current Bot Info:')

            .setDescription("Discord Compiler Bot\n"
                + "Developed by Headline#9999 (Michael Flaherty)\n"
                + "==============================\n"
                + "[Invitation link](" + invitelink + ")"
                + "\n[Vote for us!](" + votelink + ")"
                + "\n[GitHub Repository](" + githublink + ")"
                + "\n[Statistics Tracker](" + statslink + ")"
                + "\n==============================\n")

            .setColor(0x00FF00)

            .addField("Total Users", this.formatNumber(playercount), true)
            .addField("Total Servers", this.formatNumber(guildcount), true)
            .addField("CPU Usage", this.formatNumber(cpuusage.toFixed(2) + "%"), true)
            .addField("Memory Usage", this.formatNumber(memusage.toFixed(2)) + "MB", true)
            .addField("Average Ping", this.client.ws.ping.toFixed(0) + "ms", true)
            .addField("Uptime", this.formatTime(process.uptime()), true)
            .addField("System Info:", "**Node.js Version:** " + process.version
                + "\n**Operating System:** " + os.platform, false)

            .setFooter("Requested by: " + msg.message.author.tag
                + " || powered by wandbox.org");
        
        await msg.dispatch('', embed);
    }



    /**
     * Time format
     * @param {Number} seconds
     */
    formatTime(secs) {
        let seconds = Math.floor(secs);
        let hours = Math.floor(seconds / 3600) % 24;
        let minutes = Math.floor(seconds / 60) % 60;
        let seconds2 = seconds % 60;
        return [hours, minutes, seconds2]
            .map(v => v < 10 ? "0" + v : v)
            .filter((v, i) => v !== "00" || i > 0)
            .join(":");
    }

    /**
     * Formats a number in a readable fashion
     * @param {Number} num;
     */
    formatNumber(num) {
        return num.toString().replace(/(\d)(?=(\d{3})+(?!\d))/g, '$1,');
    }

    /**
     * Gets the amount of total users connected to all guilds.
     * 
     * @param {Client} client 
     */
    getUserCount(client) {
        let members = 0;
        client.guilds.cache.forEach(guild => {
            members += guild.memberCount;
        });
        return members;
    }

    /**
     * Displays the help information for the given command
     *
     * @param {CompilerCommandMessage} message
     */
    async help(message) {
        const embed = new MessageEmbed()
            .setTitle('Command Usage')
            .setDescription(`*${this.description}*`)
            .setColor(0x00FF00)
            .addField('View bot info', `${this.toString()}`)
            .setThumbnail('https://imgur.com/TNzxfMB.png')
            .setFooter(`Requested by: ${message.message.author.tag}`)
        return await message.dispatch('', embed);
    }
}
