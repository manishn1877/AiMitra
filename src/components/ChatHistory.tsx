import { MessageSquare, Plus, Trash2 } from 'lucide-react'
import { cn } from '../lib/utils'
import { useChatStore } from '../lib/chatStore'

export function ChatHistory({ onOpen }: { onOpen: () => void }) {
  const conversations = useChatStore((s) => s.conversations)
  const activeId = useChatStore((s) => s.activeId)
  const startNewChat = useChatStore((s) => s.startNewChat)
  const selectChat = useChatStore((s) => s.selectChat)
  const deleteChat = useChatStore((s) => s.deleteChat)

  return (
    <div className="mt-4 border-t border-slate-200 pt-3 dark:border-slate-700">
      <div className="mb-2 flex items-center justify-between px-3">
        <span className="text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">
          Chat history
        </span>
        <button
          type="button"
          onClick={() => {
            startNewChat()
            onOpen()
          }}
          className="btn-ghost p-1.5"
          aria-label="New chat"
          title="New chat"
        >
          <Plus className="h-4 w-4" />
        </button>
      </div>

      {conversations.length === 0 ? (
        <p className="px-3 text-xs text-slate-500 dark:text-slate-400">
          Your conversations will appear here.
        </p>
      ) : (
        <ul className="space-y-1">
          {conversations.map((c) => (
            <li key={c.id} className="group relative">
              <button
                type="button"
                onClick={() => {
                  selectChat(c.id)
                  onOpen()
                }}
                className={cn(
                  'flex w-full items-center gap-2 rounded-lg px-3 py-2 pr-9 text-left text-sm transition-colors',
                  c.id === activeId
                    ? 'bg-brand-50 text-brand-700 dark:bg-brand-900/30 dark:text-brand-300'
                    : 'text-slate-600 hover:bg-slate-100 dark:text-slate-400 dark:hover:bg-slate-800',
                )}
                title={c.title}
              >
                <MessageSquare className="h-4 w-4 flex-shrink-0" aria-hidden="true" />
                <span className="truncate">{c.title}</span>
              </button>
              <button
                type="button"
                onClick={() => {
                  if (window.confirm(`Delete "${c.title}"?`)) deleteChat(c.id)
                }}
                className="absolute right-1 top-1/2 -translate-y-1/2 rounded p-1.5 text-slate-400 opacity-0 hover:bg-slate-200 hover:text-red-600 focus:opacity-100 group-hover:opacity-100 dark:hover:bg-slate-700"
                aria-label={`Delete chat ${c.title}`}
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
