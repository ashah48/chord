import {execFile} from 'node:child_process';
import {homedir} from 'node:os';
import path from 'node:path';
import {DatabaseSync} from 'node:sqlite';
import {promisify} from 'node:util';

const run = promisify(execFile);
const epoch = Date.UTC(2001, 0, 1);

export const messagesSendScript = `
on run argv
    set recipientAddress to item 1 of argv
    set messageText to item 2 of argv
    tell application "Messages"
        set accountToUse to first account whose service type is iMessage
        set recipientToUse to participant recipientAddress of accountToUse
        send messageText to recipientToUse
    end tell
end run
`;

export async function macMessages(method, args, {
    enabled = process.env.CHORD_MACOS_MESSAGES === '1',
    database = path.join(homedir(), 'Library/Messages/chat.db'),
    send = (address, text) => run('/usr/bin/osascript', ['-e', messagesSendScript, address, text], {timeout: 30_000}),
} = {}) {
    if (!enabled) {
        if (method === 'GetStatus') return {map: false, connectivity_detail: 'Enable Messages when starting the Mac backend'};
        if (method === 'ListThreads' || method === 'ListContacts') return [];
        throw new Error('Messages is disabled');
    }
    if (method === 'ListContacts') return [];
    let db;
    try {
        db = new DatabaseSync(database, {readOnly: true, timeout: 1500});
    } catch {
        const detail = 'Cannot read Messages. Grant Full Disk Access to the terminal running Chord, then restart it.';
        if (method === 'GetStatus') return {map: false, connectivity_detail: detail};
        throw new Error(detail);
    }
    try {
        if (method === 'GetStatus') {
            db.prepare('SELECT 1 FROM chat LIMIT 1').get();
            return {map: true, connectivity_detail: 'Reading the macOS Messages database'};
        }
        const participants = db.prepare(`
            SELECT DISTINCT handle.id
            FROM chat_handle_join JOIN handle ON handle.ROWID = handle_id
            WHERE chat_id = ? ORDER BY handle.id
        `);
        function recipient(chat) {
            const handles = participants.all(chat.id);
            const address = handles[0]?.id;
            if (chat.style === 45 && chat.service === 'iMessage' &&
                handles.length === 1 && chat.address === address &&
                /^[+0-9][0-9]{2,}$|^[A-Za-z0-9_.+%-]+@[A-Za-z0-9.-]+$/.test(address)) return address;
        }
        if (method === 'SendToThreadChecked') {
            const [key, text, confirmedGroup] = args;
            const chat = db.prepare(`
                SELECT ROWID AS id, chat_identifier AS address, service_name AS service, style
                FROM chat WHERE ROWID = ?
            `).get(Number(String(key).match(/^mac-(\d+)$/)?.[1] || 0));
            const address = chat && recipient(chat);
            if (!address || confirmedGroup || typeof text !== 'string' || !text.trim() || text.includes('\0')) {
                throw new Error('Chord can send only to an existing one-to-one iMessage conversation.');
            }
            await send(address, text);
            return {submitted: true};
        }
        // The Messages database stays read-only. Opening a thread in Chord
        // does not mark it read in Apple Messages.
        if (method === 'MarkThreadRead') return false;
        if (method !== 'ListThreads') throw new Error('This action is unavailable for macOS Messages.');

        const limit = Math.min(500, Math.max(1, Number(args[0]) || 200));
        const chats = db.prepare(`
            SELECT c.ROWID AS id, c.chat_identifier AS address, c.service_name AS service,
                c.style, c.display_name AS name, max(m.date) / 1000000 AS latest
            FROM chat c JOIN chat_message_join cm ON cm.chat_id = c.ROWID
            JOIN message m ON m.ROWID = cm.message_id
            WHERE c.service_name = 'iMessage'
            GROUP BY c.ROWID ORDER BY latest DESC, c.ROWID DESC LIMIT ?
        `).all(limit);
        const messages = db.prepare(`
            SELECT m.ROWID AS id, m.text, m.date / 1000000 AS date, m.is_from_me, h.id AS sender
            FROM chat_message_join cm JOIN message m ON m.ROWID = cm.message_id
            LEFT JOIN handle h ON h.ROWID = m.handle_id
            WHERE cm.chat_id = ? AND m.text IS NOT NULL AND m.text <> ''
            ORDER BY m.date DESC, m.ROWID DESC LIMIT 50
        `);
        return chats.map(chat => {
            const handles = participants.all(chat.id).map(handle => handle.id);
            const timestamp = value => new Date(epoch + value).toISOString();
            return {
                key: 'mac-' + chat.id,
                name: chat.name || handles.join(', ') || chat.address,
                last_ts: timestamp(chat.latest),
                recipients: handles,
                is_group: chat.style !== 45 || handles.length !== 1,
                reply_ready: !!recipient(chat),
                unread: false,
                messages: messages.all(chat.id).reverse().map(message => ({
                    handle: String(message.id),
                    body: message.text,
                    timestamp: timestamp(message.date),
                    outgoing: !!message.is_from_me,
                    sender: message.is_from_me ? 'You' : message.sender,
                    address: message.sender,
                })),
            };
        });
    } finally {
        db.close();
    }
}
