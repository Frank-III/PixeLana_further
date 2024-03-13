// models/player.ts
export interface PlayerInput {
  name: string;
  avatar: string;
  pubKey: string;
}

export interface Player {
  socketId: string;
  name: string;
  avatar: string;
  isHost: boolean;
  pubKey: string;
};