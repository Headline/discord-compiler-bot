import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'
import DiscordMessageMenu from '../utils/DiscordMessageMenu'
import { Constants } from 'discord.js';

export default class ShardsCommand extends CompilerCommand {
    /**
     *  Creates the botinfo command
     * 
     * @param {CompilerClient} client
     */    
    constructor(client) {
        super(client, {
            name: 'shards',
            description: 'Shows a list of all shards',
            developerOnly: true
        });
    }

    /**
     * Function which is executed when the command is requested by a user
     *
     * @param {CompilerCommandMessage} msg
     */
    async run(msg) {
        let shardStats = await this.client.shard.broadcastEval(`
                [
                    this.guilds.cache.size,
                    this.ws.ping.toFixed(0),
                    this.ws.status,
                    this.shard.ids,
                ];
        `);

        let items = [];
        for (const shard of shardStats) {
            let str = '';
            str += `Shard ID: ${shard[3][0]}\n`;
            str += `Shard Ping: ${shard[1]}ms\n`;
            str += `Shard Status: ${this.statusToString(shard[2])}`
            items.push(str);
        }
        let menu = new DiscordMessageMenu(msg.message, 'Shard List', 0x046604, 1);
        menu.setNumbered(false);
        menu.buildMenu(items);

        try {
            await menu.displayPage(0);
        }
        catch (error) {
            msg.replyFail('Error with menu system, am I missing permissions?\n' + error);
        }
    }

    /**
     * Converts a Discord Constants.Status to it's name as string
     * 
     * @param {number} status 
     */
    statusToString(status) {
        switch (status) {
            case Constants.Status.CONNECTING:
                return "CONNECTING";
            case Constants.Status.DISCONNECTED:
                return "DISCONNECTED";
            case Constants.Status.IDLE:
                return "IDLE";
            case Constants.Status.NEARLY:
                return "NEARLY";
            case Constants.Status.READY:
                return "READY";
            case Constants.Status.RECONNECTING:
                return "RECONNECTING";
            default:
                throw new Error("Invalid Status code");
        }
    }
}
