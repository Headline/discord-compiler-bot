/**
 * Internal timer helper that expires old help command outputs
 */
export default class MessageTimeout {
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
        try {
            await message.reactions.cache.forEach(async (reaction) => {
                reaction.remove(message.author);
            });
        }
        catch (err)
        {
            // We failed out here, there's not much to do other then silenty die.
            // Rethrowing an error here won't propegate up any further...
        }

        try {
            collector.stop();    
        }
        catch (err) {
            console.log('Collector stop error: ' + err.message);
        }
    }

    restart() {
        clearTimeout(this.timeout);
        this.start();
    }
}