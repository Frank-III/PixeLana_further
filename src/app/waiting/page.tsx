'use client';
import NavBar from "@/components/navBar";
import { Button } from "@/components/ui/button";
import { Room, type User } from "@/components/waitRoom";
import { useSocketAuth } from "@/contexts/SocketAuthContext";
import { redirect } from "next/navigation";
import { useRouter } from "next/router";
import { useEffect, useMemo, useState } from "react";


export default function WaitRoom() {

  const [users, setUsers] = useState<User[]>([]);
  const {socket, socketId} = useSocketAuth();

  useEffect(() => {
    if (!socket) {
      redirect("/")
    }
    if (socket) {
      socket.emit('getPlayers'); // Request the list of players
  
      socket.on('updatePlayers', (players) => {
        console.log("players", players); 
        setUsers(players);
      });
    }
    return () => {
      if (socket) {
        socket.off('updatePlayers');
      }
    };
  }, [socket])

  // const isHost = useMemo(() => users.filter((user) => user.isHost)[0].id === socketId, [])

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <NavBar />
      <div className="flex flex-col h-[calc(100%-80px)] w-full items-center justify-center">
        <Room users={users}/> 
      </div>
      <Button className="w-[500px] h-[100px] rounded-xl italic ring-8 ring-orange-600 ring-offset-3 ring-offset-black hover:bg-[#f7d726] text-[64px] flex items-center justify-center " disabled={true}>
      <div className="rounded-full overflow-hidden mr-1" >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" height="80" width="80"><g transform="translate(0,0)" ><path d="M329.8 235.69l62.83-82.71 42.86 32.56-62.83 82.75zm-12.86-9.53l66.81-88-45-34.15-66.81 88zm-27.48-97.78l-19.3 39.57 57-75-42.51-32.3-36.24 47.71zm-20.74-73.24l-46.64-35.43-42 55.31 53.67 26.17zm107 235.52l-139-102.71-9.92.91 4.56 2 62.16 138.43-16.52 2.25-57.68-128.5-40-17.7-4-30.84 39.41 19.42 36.36-3.33 17-34.83-110.9-54.09-80.68 112.51L177.6 346.67l-22.7 145.62H341V372.62l35.29-48.93L387 275.77z" fill="#000000" fill-opacity="1"/></g></svg>
      </div>
      {"Let's Go!"}
      </Button>
    </main>
  )
}