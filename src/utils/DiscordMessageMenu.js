import { Message, MessageEmbed } from 'discord.js'
import MessageTimeout from './MessageTimeout'
import CompilerCommandMessage from '../commands/utils/CompilerCommandMessage';

/**
 * Discord Embed Menu helper class to make your life easer. It 
 * allows for on-the-fly menus, and is nearly effortless.
 */
export default class DiscordMessageMenu {

    /**
     * Constructor which builds the menu & initializes it for basic use.
     * 
     * @param {Message} authormessage 
     * @param {String} title 
     * @param {Number} color 
     */
    constructor(authormessage, title, color, displaycount, description) {
        this.menu = [];
        this.page = 0;
        this.left = '◀';
        this.stop = '🛑';
        this.right = '▶';
        this.authormessage = authormessage;
        this.message = null;
        this.displaycount = displaycount;
        this.title = title;
        this.color = color;
        this.targetid = authormessage.author.id;
        this.numbered = true;
        this.timeout = null;
        this.collector = null;
        this.description = description;
    }

    /**
     * Determines if pagination should be numbered (1) ... 2) ...) or not.
     * 
     * @param {boolean} numbered 
     */
    setNumbered(numbered) {
        this.numbered = numbered;
    }

    /**
     * Builds the menu items & formats them. This must be called
     * before the call to displayPage().
     * 
     * @param {array} items 
     */
    buildMenu(items) {
        items.forEach((element, i) => {
            if (this.numbered)
                this.menu.push('**' + (i + 1) + ')** \t*' + element + '*');
            else // TODO: should this still be bold and stuff? Configuration options maybe idk
                this.menu.push(element);
        });

        // reset page to 0
        this.page = 0;
    }

    /**
     * Returns the highest acceptable page that can be passed into
     * displayPage(). For a three page menu, this will return 2
     */
    getMaxPage() {
        return Math.ceil(this.menu.length / this.displaycount) - 1;
    }

    /**
     * Interal callback for displayPage(). Do not use.
     * 
     * @param {Message} result 
     */
    async handleMessage(result) {
        try {
            if (!result.reactions.resolve(this.stop)) {
                await result.react(this.left);
                await result.react(this.stop);
                await result.react(this.right);
            }

            // used after creation of collector to determine whether or not
            // this is the first call to handleMessage()
            let first = this.collector == null;

            // Reactions
            this.collector = result.createReactionCollector((reaction, user) => {
                if (this.targetid == user.id
                    && (reaction.emoji.name === this.left
                        || reaction.emoji.name == this.stop
                        || reaction.emoji.name == this.right)) {
                    this.collectionuser = user;
                    return true;
                }
                return false;
            }
            ).once("collect", async (reaction) => {
                try {
                    const chosen = reaction.emoji.name;
                    if (chosen == this.left) {
                        if (this.page > 0)
                            this.displayPage(--this.page)
                        else
                            this.displayPage(this.page)
                    }
                    else if (chosen == this.right) {
                        if (this.page + 1 > this.getMaxPage())
                            this.displayPage(this.page)
                        else
                            this.displayPage(++this.page);
                    }
                    else if (chosen == this.stop) {
                        await result.reactions.removeAll();
    
                        this.collector.stop();
                        this.timeout.stop();
                        return;
                    }
                    this.timeout.restart();
                    await reaction.users.remove(this.collectionuser);    
                }
                catch(err) {
                    let msg = new CompilerCommandMessage(this.message);
                    msg.replyFail(`Menu failure: ${err.message}\nAm I missing the "Manage Messages" permission?`);
                    this.collector.stop();
                    this.timeout.stop();
                }
            });

            if (first) {
                this.timeout = new MessageTimeout(result, this.collector, 30);
                this.timeout.start();
            }
        }
        catch (error) {
            throw (error); // throw to higher level
        }
    }

    /**
     * Displays the menu from the page number specified.
     * To start from page 1, pass 0.
     * 
     * @param {number} page 
     */
    async displayPage(page) {
        // Pagination building
        let start = page * this.displaycount;
        let end = start + this.displaycount;
        let items = this.menu.slice(start, end);

        // put every item on it's own line
        let output = "";
        items.forEach(element => {
            output += element + '\n';
        });

        // Message dispatch
        const embed = new MessageEmbed()
            .setTitle(this.title)
            .setColor(this.color)
            .setDescription((this.description)?this.description+"\n\n" + output:output)
            .setFooter("Requested by: " + this.authormessage.author.tag 
            + ' | page: ' + (this.page + 1) + '/' + (this.getMaxPage() + 1));


        if (this.authormessage.guild != null)
            embed.setThumbnail(this.authormessage.guild.iconURL())

        try {
            if (!this.message) { // we haven't already sent one, so send()
                this.message = await this.authormessage.channel.send(embed);
               
                // prevent multi-page elements from displaying for a single paged item
                if (this.getMaxPage() > 0)
                    await this.handleMessage(this.message);
            }
            else { // we *did* send one, so edit()
                await this.message.edit(embed);
                await this.handleMessage(this.message);
            }

        }
        catch (error) {
            throw (error); // give to higher level for handling
        }
    }
}
