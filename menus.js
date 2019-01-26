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
        if (!result.reactions.find((x) => x.emoji.name == that.stop)) {
            result.react(that.left).then(result => { // left first
                result.message.react(that.stop).then(result => { // then stop
                    result.message.react(that.right).then((result) => { 
                    }).catch(); // then right
               }).catch();
            }).catch();
        }

        // used after creation of collector to determine whether or not
        // this is the first call to handleMessage()
        let first = that.collector == null;

        // Reactions
        that.collector = result.createReactionCollector((reaction, user) => {
            if  (that.targetid == user.id
            && (reaction.emoji.name === that.left
            || reaction.emoji.name == that.stop
            || reaction.emoji.name == that.right)) {
                that.collectionuser = user;
                return true;
            }
            return false;
        }
        ).once("collect", reaction => {
            const chosen = reaction.emoji.name;
            if (chosen == that.left) {
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
            else if (chosen == that.stop) {
                result.clearReactions().then((r) => that.collector.stop());
                that.timeout.stop();
                return;
            }
            that.timeout.restart();
            reaction.remove(that.collectionuser).catch();
        });

        if (first) {
            that.timeout = new MessageTimeout(result, that.collector);
            that.timeout.start();
        }
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
        .setFooter("Requested by: " + this.authormessage.author.tag + ' | page: ' + (this.page+1) + '/' + (this.getMaxPage()+1));


        if (this.authormessage.guild != null)
            embed.setThumbnail(this.authormessage.guild.iconURL)

        if (!this.message) { // we haven't already sent one, so send()
            let that = this;
            this.message = this.authormessage.channel.send(embed)
            .then(result => that.handleMessage(result, that))
            .catch();
        }
        else { // we *did* send one, so edit()
            let that = this;
            this.message.edit(embed)
            .then(result => that.handleMessage(result, that))
            .catch();
        }
    }
}

class MessageTimeout {
    constructor(message, collector, delay) {
        this.message = message;
        this.collector = collector
        this.delay = delay;
        this.timeout = null;
    }

    start() {
        this.timeout = setTimeout(this.run, 30 * 1000, this.message, this.collector);
    }

    stop() {
        clearTimeout(this.timeout);
    }

    run(message, collector) {
        message.clearReactions().then(result => collector.stop()).catch();
    }

    restart() {
        clearTimeout(this.timeout);
        this.start();
    }
}
module.exports = DiscordMessageMenu;