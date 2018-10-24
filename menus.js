const Discord = require('discord.js');

/**
 * Discord Embed Menu helper class to make your life easer. It 
 * allows for on-the-fly menus, and is nearly effortless.
 */
class DiscordMessageMenu {

    /**
     * Constructor which builds the menu & initializes it for basic use.
     * 
     * @param {Discord.Message} authormessage 
     * @param {String} title 
     * @param {Number} color 
     */
    constructor(authormessage, title, color, displaycount) {
        this.menu = [];
        this.page = 0;
        this.left = '◀';
        this.right = '▶';
        this.authormessage = authormessage;
        this.message = null;
        this.displaycount = displaycount;
        this.title = title;
        this.color = color;
        this.targetid = authormessage.author.id;
        this.numbered = true;
    }

    /**
     * Determines if pagination should be numbered (1) ... 2) ...) or not.
     * 
     * @param {Boolean} numbered 
     */
    setNumbered(numbered) {
        this.numbered = numbered;
    }

    /**
     * Builds the menu items & formats them. This must be called
     * before the call to displayPage().
     * 
     * @param {Array} items 
     */
    buildMenu(items) {
        items.forEach((element, i) => {
            if (this.numbered)
                this.menu.push('**' + (i+1) + ')** \t*' + element + '*');
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
        return Math.ceil(this.menu.length/this.displaycount) - 1;
    }

    /**
     * Interal callback for displayPage(). Do not use.
     * 
     * @param {Discord.Message} result 
     * @param {DiscordMessageMenu} that 
     */
    handleMessage(result, that) {
        that.message = result;
        that.message.react(that.left).then(result => { // left first
            that.message.react(that.right).catch(console.log) // then right
        }).catch(console.log);

        // Reactions
        const collector = that.message.createReactionCollector((reaction, user) =>
        that.targetid == user.id
            && (reaction.emoji.name === that.left
            || reaction.emoji.name == that.right)
        ).once("collect", reaction => {
            const chosen = reaction.emoji.name;
            if (chosen === that.left) {
                if (that.page > 0)
                    that.displayPage(--that.page)
                else
                    that.displayPage(that.page)
            }
            else if (chosen == that.right) {
                if (that.page + 1 > that.getMaxPage())
                    that.displayPage(that.page)
                else
                    that.displayPage(++that.page);
            }
            that.message.clearReactions();
            collector.stop();
        });
    }

    /**
     * Displays the menu from the page number specified.
     * To start from page 1, pass 0.
     * 
     * @param {Number} page 
     */
    displayPage(page) {
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
        const embed = new Discord.RichEmbed()
        .setTitle(this.title)
        .setColor(this.color)
        .setDescription(output)
        .setThumbnail(this.authormessage.guild.iconURL)
        .setFooter("Requested by: " + this.authormessage.author.tag + ' | page: ' + (this.page+1) + '/' + (this.getMaxPage()+1));


        if (!this.message) { // we haven't already sent one, so send()
            let that = this;
            this.message = this.authormessage.channel.send(embed)
            .then(result => that.handleMessage(result, that))
            .catch(console.log);
        }
        else { // we *did* send one, so edit()
            let that = this;
            this.message.edit(embed)
            .then(result => that.handleMessage(result, that))
            .catch(console.log);
        }
    }
}

module.exports = DiscordMessageMenu;