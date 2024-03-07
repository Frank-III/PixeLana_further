'use client'
import React, {
  createContext,
  useContext,
  useEffect,
  useState,
  FC,
  ReactNode,
} from "react";
import { useSocketAuth } from "./SocketAuthContext";
import { type User } from "@/components/waitRoom";
import { toast } from "sonner";
import { useRouter } from "next/navigation";

interface GameState {
  players: Array<User>;
  isHost: boolean;
  playerIdx: number;
  round: number;
  gameState:
    | "none"
    // | "init"
    | "waitingForPlayers"
    | "waitingForPrompt"
    | "waitingForDraw"
    | "ended";
  leaderBoard: Array<[string, string]>;
}

const defaultGameState: GameState = {
  players: [],
  isHost: false,
  playerIdx: -1,
  round: 0,
  gameState: "none",
  leaderBoard: [],
};

const GameStateContext = createContext<GameState>(defaultGameState);

export const useGameState = () => useContext(GameStateContext);

export const GameStateProvider: FC<{ children: ReactNode }> = ({
  children,
}) => {
  const [gameState, setGameState] = useState<GameState>(defaultGameState);
  const { socket } = useSocketAuth();
  const router = useRouter();

  useEffect(() => {
    if (socket) {
      // socket.on("gameState", (newGameState: GameState) => {
      //   setGameState(newGameState);
      // });
      socket.on('disconnect', () => {
        setGameState(defaultGameState)
      })

      socket.on("goBackLobby", () => {
        setGameState((prev) => ({
          ...prev,
          uploadedImgs: [],
          gameState: "waitingForPlayers",
        }));
      })

      socket.on("updatePlayers", (players: Array<User>) => {
        setGameState((prev) => ({
          ...prev,
          players,
          isHost: prev.isHost || players.length === 1,
          playerIdx: prev.playerIdx > -1 ? prev.playerIdx : players.length -1,
          gameState: "waitingForPlayers",
        }));
      });

      socket.on("updateLeaderBoard", (leaderBoard: Array<[string, string]>) => {
        setGameState((prev) => ({
          ...prev,
          leaderBoard,
        }));
      });

      socket.on('promptStart', () => {
        setGameState((prev) => ({ ...prev, gameState: "waitingForPrompt" }));
        router.replace("/prompt")
      })

      socket.on('promptFinished', () => {
        setGameState((prev) => ({ ...prev, gameState: "waitingForDraw" }));
        router.replace("/round")
      })

      socket.on('goBackLobby', () => {
        setGameState((prev) => ({
          ...prev,
          round: 0,
          gameState: "waitingForPlayers",
        }));
        router.replace("/")
      })
      // socket.on("endGame", () => {
      //   setGameState((prev) => ({ ...prev, gameState: "ended" }));
      // });

      // socket.on("bestImage", (playerId: string, exploreUrl) => {
      //   toast.success(
      //     " The best image come from: " +
      //       playerId +
      //       " and the url is: " +
      //       exploreUrl,
      //   );
      // });

      return () => {
        socket.off("gameState");
        socket.off("updatePlayers");
        socket.off("updateLeaderBoard");
        socket.off("promptStart");
        socket.off("promptFinished");
        socket.off("goBackLobby");
        // socket.off("prompt");
        // socket.off("draw");
        // socket.off("endGame");
      };
    }
  }, [socket]);

  return (
    <GameStateContext.Provider value={gameState}>
      {children}
    </GameStateContext.Provider>
  );
};
