// models/player.ts

export type Player = {
    socketId: string;
    name: string;
    avatar: string;
    isHost: boolean;
    pubKey: string;
  };