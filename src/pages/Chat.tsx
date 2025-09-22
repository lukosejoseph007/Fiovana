import React from 'react'
import ChatInterface from '../components/chat/ChatInterface'

const Chat: React.FC = () => {
  return (
    <div className="h-[calc(100vh-12rem)]">
      <ChatInterface className="h-full" />
    </div>
  )
}

export default Chat
