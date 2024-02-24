import { Server, Socket } from 'socket.io';
import { createServer } from 'http';
import { Player } from './models/player';
import { Prompt } from './models/prompt';

const httpServer = createServer();
const io = new Server(httpServer, {
    cors: {
        origin: '*',
        methods: ['GET', 'POST']
    }
});

let players: Record<string, Player> = {}; // publicKey: Player
let order = 0; // Keep track of the number of players
let startingPrompts: Prompt[] = [];
let gameStarted = false;

// Player connects
io.on('connect', (socket) => {
    // Add player to memory when they connect to lobby
    socket.on('addPlayer', (name, avatar, isHost, publicKey) => {
        if (players[publicKey]) {
            socket.emit('addPlayerError', `Public key ${publicKey} is already in use.`);
            return;
        }
        
        let player = { id: socket.id, name, avatar, isHost, order, publicKey };
        players[publicKey] = player;
        order++;
        
        console.log(`User ${socket.id} connected. Total players: ${Object.keys(players).length}, Order: ${order}`);
        io.emit('updatePlayers', Object.values(players));
    });


    // Player disconnects
    socket.on('disconnect', () => {
        let player = Object.values(players).find(player => player.id === socket.id);
        if (!player) {
            console.log(`User ${socket.id} not found.`);
            return;
        }

        delete players[player.publicKey];
        console.log(`User ${socket.id} disconnected. Total players: ${Object.keys(players).length}`);
        io.emit('updatePlayers', Object.values(players));
    });


    // Provide current list players to the new player
    socket.on('getPlayers', () => {
        socket.emit('updatePlayers', Object.values(players));
        console.log(`User ${socket.id} requested players.`);
    });

    
    // Listen for host starting the game
    socket.on('startGame', () => {
        gameStarted = true;
        io.emit('gameStart');

        // Start a timer for 60 seconds
        setTimeout(() => {
            // If not all players have submitted their prompts when the timer runs out,
            // submit an empty prompt for them and move on to the next stage of the game
            if (startingPrompts.length !== Object.keys(players).length) {
                for (let publicKey in players) {
                    if (!startingPrompts.find(prompt => prompt.player.publicKey === publicKey)) {
                        let prompt: Prompt = { player: players[publicKey], text: '' };
                        startingPrompts.push(prompt);
                    }
                }

                io.emit('gameDraw');
            }
        }, 60000); // 60 seconds
    });  

    
    // Listen for player submitting their prompt's text
    socket.on('submitPrompt', (publicKey, promptText) => {
        let player = players[publicKey];
        let prompt: Prompt = { player, text: promptText };
        startingPrompts.push(prompt);
    
        console.log(`User ${socket.id} submitted prompt: ${promptText}`);

        // Check if all players have submitted their prompts
        if (startingPrompts.length === Object.keys(players).length) {
            // Move onto drawing the prompts
            io.emit('gameDraw');
        }
    });
});

const PORT = process.env.PORT || 3001;
httpServer.listen(PORT, () => console.log(`Server running on port ${PORT}`));