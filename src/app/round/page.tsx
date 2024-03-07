'use client';
import { useSocketAuth } from "@/contexts/SocketAuthContext";
import { useEffect, useState } from "react";
import Image from "next/image";
import NavBar from "@/components/navBar";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { revalidatePath } from "next/cache";
import { cn } from "@/lib/utils";
import { useGameState } from "@/contexts/GameStateProvider";
import { play } from "@/lib/iconSvgs";
import { useRouter } from "next/navigation";
import { useAction } from "@/lib/useAction";

function FinishDialog({open}: {open: boolean}) {
  return (
  <Dialog open={open}>
    <DialogContent className="bg-secondary">
      <DialogHeader>
        <DialogTitle className="font-sans text-white">Waiting for other users to finish</DialogTitle>
        <DialogDescription className="font-sans text-white">
          Please wait for the other users to finish their turn!
        </DialogDescription>
      </DialogHeader>
    </DialogContent>
  </Dialog>
  )
}

 async function query(prompt: string) {
    // console.log(process.env.NEXT_SDXL_API_KEY)
    const response = await fetch(
      "https://api-inference.huggingface.co/models/stabilityai/stable-diffusion-xl-base-1.0",
      {
        headers: { Authorization: process.env.SDXL_API_KEY || "" },
        method: "POST",
        body: JSON.stringify({ inputs: prompt }),
      }
    );
    const result = await response.blob();
    return result;
}

const buttonStyle="rounded-xl italic ring-[5px] ring-orange-600 hover:bg-[#f7d726] text-shadow-md"
const inputStyle ="flex-1 border ring-orange-600 ring-[5px] rounded-lg focus-visible:ring-emerald-600 focus-visible:ring-[5px]"

export default function Game() {

  const router = useRouter()
  const {socket} = useSocketAuth();
  const {submitRoundInfo} = useAction();
  const {playerIdx} = useGameState();
  // if image Round (for a certain user)
  const [isImage, setIsImage] = useState(false);
  // received content, either image or story
  const [receivedContent, setContent] = useState<string | null>(null);
  // user typed prompt to submit
  const [story, setStory] = useState<string>("");
  const [aiPrompt, setAIPrompt] = useState<string>("");
  // the AI image
  const [aiImage, setAiImage] = useState<string>();
  // is the AI generating the image
  const [generating, setGenerating] = useState(false);
  // has the user submitted their content
  const [submitted, setSubmitted] = useState(false);
  // duration of the round
  const [timeLeft, setTimeLeft] = useState(60); 

  const submitInfo= (info: string) => {
    submitRoundInfo(playerIdx.toString(), info);
    setSubmitted(true);
  }

  useEffect(() => {
    // Exit early when we reach 0
    if (!timeLeft) {
      if (isImage) {
        submitInfo(story);
      } else {
        submitInfo(aiImage || "/404.png");
      }
    };

    // Save intervalId to clear the interval when the component re-renders
    const intervalId = setInterval(() => {
      setTimeLeft(timeLeft - 1);
    }, 1000);

    // Clear interval on re-render to avoid memory leaks
    return () => clearInterval(intervalId);
    // Add timeLeft as a dependency to re-run the effect
    // when we update it
  }, [timeLeft]);


  useEffect(() => { 
    if(socket) {
      socket.emit('getRoundInfo', playerIdx)

      socket.on('roundInfo', (content: {type: "image" | "prompt", data: string}) => {
        setSubmitted(false);
        setIsImage(content.type === "image");
        setContent(content.data);
        setTimeLeft(60);
      })

      socket.on('roundFinished', (data) => {
        socket.emit('getRoundInfo', playerIdx);
      })

      socket.on('gameFinished', () => {
        router.push('/end')
      })
    }
  }, [socket])

  const generate = async (e) => {
    e.preventDefault(); 
    setGenerating(true);
    const blob = await query(aiPrompt);
    const url = URL.createObjectURL(blob);
    setGenerating(false);
    setAiImage(url);
  }


  return (
<main className="flex min-h-screen flex-col items-center justify-center p-24">
  <NavBar />
  <div className="flex flex-col items-center justify-center w-full z-10 " style={{ height: 'calc(100% - 74px)' }}>
  <div className="flex flex-col items-center justify-center min-h-[80%] min-w-[80%] w-[80%] h-[80%] bg-[#370C59] rounded-lg p-5 space-y-3 border-[3px] border-gray-200">
      <h1 className="font-customs text-[50px] text-shadow-custom text-[#8dfcbc]">Cook Your Story</h1>
      {!isImage && <h1 className="text-shadow-md text-xl text-yellow-300">Prompt: {receivedContent!}</h1>}
      {/* Ensure Image component fills the container or consider a wrapper */}
      <div className="border-[5px] border-black rounded-xl w-[500px] h-[500px]">
        {isImage ? 
        ( receivedContent ? <Image src={receivedContent!} alt="image" width={500} height={500} className="w-full h-full object-contain" /> : (<></>)) : 
        (aiImage ? (<Image src={aiImage} width={500} height={500} alt="image" className="w-full h-full object-contain" />) :
        (<div className="text-white items-center justify-center w-full h-full flex">Input your Prompt to Generate Image</div>)) }
      </div>
      <h1 className={cn("text-shadow-md text-xl text-white", timeLeft < 20 && "text-yellow-300")}>Time Remaining: {timeLeft}</h1>
    <div className="w-[80%] flex space-x-5">
      {isImage ? <Input className={inputStyle} value={story} onChange={(e) => {setStory(e.currentTarget.value)}}/> : <Input className={inputStyle} value={aiPrompt} onChange={(e) => {setAIPrompt(e.currentTarget.value)}}/>}
      {!isImage && <Button className={buttonStyle} onClick={generate} disabled={generating}>Generate</Button>}
      {isImage ? <Button className={buttonStyle} onClick={() => submitInfo(story)} disabled={story === ""}>Submit</Button> : <Button className={buttonStyle} onClick={() => submitInfo(aiImage || "/404.png")} disabled={!aiImage}>Submit</Button>}
    </div>
    </div>
  </div>
  <FinishDialog open={submitted}/>
</main>
  )
}