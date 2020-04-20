import {Message, ReactionCollector} from 'discord.js'

/**
 * Internal timer helper that expires old help command outputs
 */
export default class MessageTimeout {
    /**
     * Creates a MessageTimeout which handles reaction removal and collection closure
     * 
     * @param {Message} message message to remove reactions from after delay
     * @param {ReactionCollector} collector reaction collector to close after delay
     * @param {number} delay delay in seconds
     */
    constructor(message, collector, delay) {
        this.message = message;
        this.collector = collector;
        this.delay = delay;
        this.timeout = null;
    }

    /**
     * Starts the message timeout timer
     */
    start() {
        this.timeout = setTimeout(this.run, this.delay * 1000, this.message, this.collector);
    }

    /**
     * Stops the message timeout timer
     */
    stop() {
        clearTimeout(this.timeout);
    }

    /**
     * 
     * @param {Message} message 
     * @param {ReactionCollector} collector 
     */
    async run(message, collector) {
        message.reactions.cache.forEach(async (reaction) => {
            try {
                await reaction.remove(message.author);
            }
            catch (err) {
                // We failed out here, there's not much to do other than die.
                // Rethrowing an error here won't propegate up any further...
                message.client.emit('missingPermissions', message.guild);
            }
        });

        collector.stop();
    }

    restart() {
        clearTimeout(this.timeout);
        this.start();
    }
}