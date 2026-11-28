import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'

export interface ChatMessage {
  id: string
  role: 'user' | 'assistant'
  content: string
  timestamp: number
  timing?: {
    prompt_ms: number
    generation_ms: number
    tokens_per_second: number
  }
}

export interface Conversation {
  id: string
  title: string
  createdAt: number
  updatedAt: number
  messages: ChatMessage[]
}

const MAX_CONVERSATIONS = 100
const DEFAULT_TITLE = 'New chat'

export function makeTitle(text: string): string {
  const clean = text.replace(/\s+/g, ' ').trim()
  if (!clean) return DEFAULT_TITLE
  return clean.length > 48 ? `${clean.slice(0, 47)}…` : clean
}

interface ChatState {
  conversations: Conversation[]
  /** null means "a new chat that has no messages yet". */
  activeId: string | null
  startNewChat: () => void
  selectChat: (id: string) => void
  deleteChat: (id: string) => void
  clearAll: () => void
  /** Adds a message and returns the id of the conversation it went into. */
  addMessage: (
    conversationId: string | null,
    message: Pick<ChatMessage, 'role' | 'content' | 'timing'>,
  ) => string
}

export const useChatStore = create<ChatState>()(
  persist(
    (set, get) => ({
      conversations: [],
      activeId: null,

      startNewChat: () => set({ activeId: null }),

      selectChat: (id) => {
        if (get().conversations.some((c) => c.id === id)) set({ activeId: id })
      },

      deleteChat: (id) =>
        set((state) => ({
          conversations: state.conversations.filter((c) => c.id !== id),
          activeId: state.activeId === id ? null : state.activeId,
        })),

      clearAll: () => set({ conversations: [], activeId: null }),

      addMessage: (conversationId, message) => {
        const now = Date.now()
        const full: ChatMessage = { ...message, id: crypto.randomUUID(), timestamp: now }
        const existing = get().conversations.find((c) => c.id === conversationId)

        if (!existing) {
          const conversation: Conversation = {
            id: crypto.randomUUID(),
            title: message.role === 'user' ? makeTitle(message.content) : DEFAULT_TITLE,
            createdAt: now,
            updatedAt: now,
            messages: [full],
          }
          set((state) => ({
            conversations: [conversation, ...state.conversations].slice(0, MAX_CONVERSATIONS),
            activeId: conversation.id,
          }))
          return conversation.id
        }

        const updated: Conversation = {
          ...existing,
          title:
            existing.title === DEFAULT_TITLE && message.role === 'user'
              ? makeTitle(message.content)
              : existing.title,
          updatedAt: now,
          messages: [...existing.messages, full],
        }
        set((state) => ({
          conversations: [updated, ...state.conversations.filter((c) => c.id !== existing.id)],
        }))
        return existing.id
      },
    }),
    {
      name: 'aimitra-chats',
      version: 1,
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ conversations: state.conversations, activeId: state.activeId }),
    },
  ),
)
