import CompilerCommand from './utils/CompilerCommand'
import CompilerCommandMessage from './utils/CompilerCommandMessage'
import CompilerClient from '../CompilerClient'
import DiscordMessageMenu from '../utils/DiscordMessageMenu'

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
        let list = [];
        let collection = this.client.guilds.cache.sort((g1, g2) => {
            return g2.memberCount - g1.memberCount;
        });
        collection.forEach((g) => {
            list.push(`${g.name} - ${g.memberCount} members`);
        });

        let menu = new DiscordMessageMenu(msg.message, 'Server list', 0x00FF00, 15);
        menu.buildMenu(list);

        try {
            await menu.displayPage(0);
        }
        catch (error) {
            msg.replyFail('Error with menu system, am I missing permissions?\n' + error);
        }
    }
}
