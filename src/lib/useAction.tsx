'use client';

import { useGameState } from "@/contexts/GameStateProvider";
import { useSocketAuth } from "@/contexts/SocketAuthContext";
import { useCallback } from "react";

//TODO: add a parameter: isHost to make sure he could access the host actions
export const useAction = (host= false) => {
  const { socket, disconnectSocket } = useSocketAuth();
  const { isHost, gameState } = useGameState();

  // equal leaveGame ?
  const reset = useCallback(() => {
    if (socket) {
      disconnectSocket()
    }
  }, [socket, disconnectSocket])

  const joinGame = useCallback(() => {
    if (socket) {
      socket.emit("addPlayer");
    }
  }, [socket]);

  const startGame = useCallback(() => {
    if (socket) {
      console.log(isHost,gameState) 
      socket.emit("startGame");
    }
  }, [socket]);

  const endGame = useCallback(() => {
    if (socket) {
      socket.emit("endGame");
    }
  }, [socket]);

  const submitPrompt = useCallback(
    (playerIdx: string, prompt: string) => {
      console.log("submitPrompt", playerIdx, prompt)
      if (socket) {
        socket.emit("submitPrompt", playerIdx, prompt);
      }
    },
    [socket],
  );

  const submitRoundInfo = useCallback(
    (playerIdx: string, info: string) => {
      if (socket) {
        socket.emit("submitRoundInfo", playerIdx, info);
      }
    },
    [socket],
  );

  const likeDraw = useCallback(
    (playerIdx: string, likeIdx: string) => {
      if (socket) {
        socket.emit("likeDrawing", playerIdx, likeIdx);
      }
    },
    [socket],
  );

  const backRoom = useCallback(() => {
    if (socket) {
      socket.emit("backRoom");
    }
  }, [socket]);

  return {
    reset,
    joinGame,
    startGame,
    endGame,
    submitPrompt,
    submitRoundInfo,
    likeDraw,
    backRoom,
  };
};
