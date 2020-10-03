import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'
import DiscordMessageMenu from '../utils/DiscordMessageMenu'
import log from '../log'

export default class BotInfoCommand extends CompilerCommand {
    /**
     *  Creates the botinfo command
     * 
     * @param {CompilerClient} client
     */    
    constructor(client) {
        super(client, {
            name: 'servers',
            description: 'Shows a list of all servers',
            developerOnly: true
        });
    }

    /**
     * Function which is executed when the command is requested by a user
     *
     * @param {CompilerCommandMessage} msg
     */
    async run(msg) {
        let guildsArrays = [];
        try {
            guildsArrays = await this.client.shard.broadcastEval(`
                this.guilds.cache.array();
            `);
        }
        catch (e) {
            log.warn(`ServersCommand#run -> ${e}`);
        }

        // Join each guildsArrays (per shard guild list) into a single array
        // containing a list of all guilds the bot is in
        let servers = [].concat.apply([], guildsArrays);

        let collection = servers.sort((g1, g2) => {
            return g2.memberCount - g1.memberCount;
        });

        let list = [];
        collection.forEach((g) => {
            list.push(`${g.name} - ${g.memberCount} members`);
        });

        let menu = new DiscordMessageMenu(msg.message, 'Server list', 0x046604, 15);
        menu.buildMenu(list);

        try {
            await menu.displayPage(0);
        }
        catch (error) {
            msg.replyFail('Error with menu system, am I missing permissions?\n' + error);
        }
    }
}
