import { like } from './../src/lib/iconSvgs';
import { Server, Socket } from 'socket.io';
import { createServer } from 'http';
import { Player } from './models/player';
import mint from './mint';
import { rotateRecord } from './utils';

const httpServer = createServer();
const io = new Server(httpServer, {
    cors: {
        origin: '*',
        methods: ['GET', 'POST']
    }
});

let players: Record<string, Player> = {}; // publicKey: Player
let totalPlayers = 0;
let round = 0;
let roundImgsOrPrompt: Array<Record<number, string>> = [];
let prevOrdImgsOrPrompt: Record<number, string> = {};
let prevGot: number = 0;
let playerIdxToPublicKey: Record<number, string> = {}; // playerIdx: publicKey
let pubKeyToPlayerIdx: Record<string, number> = {}; // playerIdx: publicKey
let socketIdToPublicKey: Record<string, string> = {}; // socketId: publicKey
let endroundRec = 0;
let endroundLike = 0;
let leaderBoard: Record<string, number> = {}; // Leaderboard of likes
let gameStarted = false;


function cleanGame() {
    round = 0;
    roundImgsOrPrompt = [];
    prevOrdImgsOrPrompt = {};
    prevGot = 0;
    socketIdToPublicKey = {};
    endroundRec = 0;
    endroundLike = 0;
    gameStarted = false;
}



io.use((socket, next) => {
    if (gameStarted) {
        return next(new Error('Game already started.'));
    }
    return next();
})

// Player connects
io.on('connect', (socket) => {
    // Add player to memory when they connect to lobby
    socket.on('addPlayer', (name, avatar, publicKey) => {
        if (players[publicKey]) {
            socket.emit('addPlayerError', `Public key ${publicKey} is already in use.`);
            return;
        }
        const isHost = Object.keys(players).length == 0; 
        let player = { socketId: socket.id, name, avatar, isHost, publicKey };
        playerIdxToPublicKey[totalPlayers] = publicKey;
        pubKeyToPlayerIdx[publicKey] = totalPlayers;
        totalPlayers++;
        players[publicKey] = player;
        socketIdToPublicKey[socket.id] = publicKey;
        leaderBoard[publicKey] = 0;
        
        
        console.log(`User ${socket.id} connected. Total players: ${Object.keys(players).length}`);
        io.emit('updatePlayers', Object.values(players));
        io.emit('updateLeaderBoard', Object.entries(leaderBoard).sort((a, b) => b[1] - a[1]));
    });

    // Player disconnects
    socket.on('disconnect', () => {
        let player = players[socketIdToPublicKey[socket.id]];
        if (!player) {
            console.log(`User ${socket.id} not found.`);
            return;
        }

        delete players[player.publicKey];
        delete socketIdToPublicKey[socket.id];
        delete leaderBoard[player.publicKey];
        totalPlayers--;
        // playerIdxToPublicKey = Object.entries(playerIdxToPublicKey).reduce((acc, [idx, publicKey]) => {
        //     if (player.publicKey === publicKey) {
        //         return acc;
        //     }

        // }, {})
        if (player.isHost) {
            gameStarted = false;
        }
        console.log(`User ${socket.id} disconnected. Total players: ${Object.keys(players).length}`);
        console.log(players);
        io.emit('updatePlayers', Object.values(players));
    });

    // Listen for host starting the game
    socket.on('startGame', () => {
        gameStarted = true;
        console.log(`Host ${socket.id} started the game.`)
        io.emit("promptStart");
    }); 
    
    
    socket.on('submitPrompt', (playerIdx, prompt) => {
        if (!roundImgsOrPrompt.length) roundImgsOrPrompt.push({});
        roundImgsOrPrompt[0][playerIdx] = prompt;
        if (Object.keys(roundImgsOrPrompt[0]).length === totalPlayers) {
            round += 1;
            prevOrdImgsOrPrompt = rotateRecord(roundImgsOrPrompt[0]);
            io.emit('promptFinished');
        }
    })

    socket.on('getRoundInfo', (playerIdx) => {
        // const curIdx = 
        const info = prevOrdImgsOrPrompt[playerIdx];
        // prevGot +=1;
        round % 2== 0 ? socket.emit('roundInfo', {type: "image", data: info} ) : socket.emit('roundInfo', {type: "prompt", data: info});

    })

    socket.on('submitRoundInfo', (playerIdx, imgOrPrompt) => {
        console.log(`User ${playerIdx} submitted ${imgOrPrompt}`)
        if (roundImgsOrPrompt.length === round) {
            roundImgsOrPrompt.push({});
        }
        roundImgsOrPrompt[round][playerIdx] = imgOrPrompt;
        if (Object.keys(roundImgsOrPrompt[round]).length === totalPlayers) {
            console.log(`round ${round} finished`)
            if (round + 1 === totalPlayers) {
                io.emit('gameFinished');
            } else {
                prevOrdImgsOrPrompt = rotateRecord(roundImgsOrPrompt[round]);
                io.emit('roundFinished');
            }
            round += 1;
        }
    })

    socket.on('getAllImgsOrPrompts', (roundRec) => {
        const roundImgOrPromp = roundImgsOrPrompt.map((imgsOrPrompt, idx) => {
            console.log(idx, (idx + roundRec) % totalPlayers)
            // console.log(imgsOrPrompt)
            return {type: idx % 2 == 0 ? "story" : "image", data: imgsOrPrompt[(idx + roundRec) % totalPlayers], idx: (idx + roundRec) % totalPlayers}
        })

        console.log(roundImgOrPromp)

        socket.emit('allImgsOrPrompts', roundImgOrPromp, roundRec+1);
        // endroundRec += 1;
    })

    socket.on('likeDrawing', async (playerIdx, likeIdx) => {
        console.log(playerIdx, likeIdx)
        const publicKey = playerIdxToPublicKey[likeIdx];
        const best = roundImgsOrPrompt[Math.abs((likeIdx - playerIdx)) % totalPlayers][likeIdx];
        leaderBoard[publicKey] = leaderBoard[publicKey] + 1;
        io.emit('updateLeaderBoard', Object.entries(leaderBoard).sort((a, b) => b[1] - a[1]))
        const data = {
            image: best,
        }
        const exploreUrl = await mint(publicKey, data)

        io.emit('bestImage', likeIdx, exploreUrl);
        endroundLike += 1
        if (endroundLike === totalPlayers) {

        } else {
        io.emit('roundImgLiked', endroundLike);
        }
        console.log(`User ${socket.id} liked ${playerIdx}`);
    })

    socket.on('backRoom', () => {
        cleanGame()
        io.emit('goBackLobby');
    })
});

const PORT = process.env.PORT || 3001;
httpServer.listen(PORT, () => console.log(`Server running on port ${PORT}`)); 