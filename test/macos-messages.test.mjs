import assert from 'node:assert/strict';
import {mkdtemp} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {DatabaseSync} from 'node:sqlite';
import test from 'node:test';
import {macMessages} from '../macos-messages.mjs';

test('reads Messages in order and only sends to a verified direct iMessage chat', async () => {
    const directory = await mkdtemp(path.join(tmpdir(), 'chord-messages-'));
    const database = path.join(directory, 'chat.db');
    const fixture = new DatabaseSync(database);
    fixture.exec(`
        CREATE TABLE chat (chat_identifier TEXT, service_name TEXT, style INTEGER, display_name TEXT);
        CREATE TABLE chat_handle_join (chat_id INTEGER, handle_id INTEGER);
        CREATE TABLE handle (id TEXT);
        CREATE TABLE message (text TEXT, date INTEGER, is_from_me INTEGER, handle_id INTEGER);
        CREATE TABLE chat_message_join (chat_id INTEGER, message_id INTEGER);
        INSERT INTO chat VALUES
            ('dad@example.org', 'iMessage', 45, NULL),
            ('group', 'iMessage', 43, 'Family'),
            ('+15550123456', 'SMS', 45, NULL);
        INSERT INTO handle VALUES ('dad@example.org'), ('+15550123456');
        INSERT INTO chat_handle_join VALUES (1, 1), (2, 1), (2, 2), (3, 2);
        INSERT INTO message VALUES
            ('hello', 800000000000000000, 0, 1),
            ('hi', 800000001000000000, 1, NULL);
        INSERT INTO chat_message_join VALUES (1, 1), (1, 2), (2, 1), (3, 2);
    `);
    fixture.close();

    const sent = [];
    const options = {enabled: true, database, send: (...args) => sent.push(args)};
    const [direct, group] = await macMessages('ListThreads', [50], options);
    assert.equal(direct.key, 'mac-1');
    assert.equal(direct.reply_ready, true);
    assert.deepEqual(direct.messages.map(message => message.body), ['hello', 'hi']);
    assert.equal(direct.messages[1].timestamp, '2026-05-09T06:13:21.000Z');
    assert.equal(group.reply_ready, false);
    await macMessages('SendToThreadChecked', ['mac-1', 'a "quoted" message', false], options);
    assert.deepEqual(sent, [['dad@example.org', 'a "quoted" message']]);
    await assert.rejects(macMessages('SendToThreadChecked', ['mac-2', 'group', true], options));
    await assert.rejects(macMessages('SendToThreadChecked', ['mac-3', 'SMS', false], options));
    assert.deepEqual(await macMessages('ListThreads', [10], {enabled: false, database}), []);
});
