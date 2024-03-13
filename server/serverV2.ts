import { PlayerInput, Player } from "./models/player";

export class GameState {

  players: Record<string, Player> = {}; // publicKey: Player
  totalPlayers = 0;
  round = 0;
  roundImgsOrPrompt: Array<Record<number, string>> = [];
  prevOrdImgsOrPrompt: Record<number, string> = {};
  prevGot: number = 0;
  playerIdxToPublicKey: Record<number, string> = {}; // playerIdx: publicKey
  pubKeyToPlayerIdx: Record<string, number> = {}; // playerIdx: publicKey
  socketIdToPublicKey: Record<string, string> = {}; // socketId: publicKey
  private endroundRec = 0;
  private endroundLike = 0;
  leaderBoard: Record<string, number> = {}; // Leaderboard of likes
  gameStarted = false;

  constructor() {
  }

  getLeaderBoard() {
    return Object.entries(this.leaderBoard).sort((a, b) => b[1] - a[1]);
  }

  resetGame() {
    this.round = 0;
    this.roundImgsOrPrompt = [];
    this.prevOrdImgsOrPrompt = {};
    this.prevGot = 0;
    this.endroundRec = 0;
    this.endroundLike = 0;
    this.gameStarted = false;
  }

  addPlayer(player: PlayerInput, socketId: string) {

    this.players[player.pubKey] = {
      ...player,
      socketId,
      isHost: Object.keys(this.players).length == 0
    };
    this.totalPlayers++;
    this.leaderBoard[player.pubKey] = 0;
    return Object.values(this.players); 
  }

  startGame() {
    this.gameStarted = true;
    return true
  }

  submitImgOrPrompt(imgOrPrompt: string, playerIdx: number) {
  }

  sendRoundInfo() {

  }

  getAllImgsOrPrompts() {

  }

  LikeImge() {

  }
}