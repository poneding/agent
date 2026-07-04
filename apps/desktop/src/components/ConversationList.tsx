import type { DesktopConversationItem } from '../types';

export function ConversationList({
  conversations,
  activeConversationId,
  onSelect,
}: {
  conversations: DesktopConversationItem[];
  activeConversationId: string | null;
  onSelect: (conversationId: string) => void;
}) {
  if (conversations.length === 0) {
    return <div className="empty-list">No conversations yet</div>;
  }

  return (
    <ul className="conversation-list" aria-label="Conversations">
      {conversations.map((conversation) => (
        <li key={conversation.conversation_id}>
          <button
            className={conversation.conversation_id === activeConversationId ? 'selected' : ''}
            type="button"
            onClick={() => onSelect(conversation.conversation_id)}
          >
            <span>{conversation.title}</span>
            {conversation.unread_count > 0 ? <strong>{conversation.unread_count}</strong> : null}
          </button>
        </li>
      ))}
    </ul>
  );
}
