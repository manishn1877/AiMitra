import test from 'node:test'
import assert from 'node:assert/strict'
import { parseMessageParts, runnableLanguage } from './messageParts'

test('plain text stays a single text part', () => {
  assert.deepEqual(parseMessageParts('hello there'), [{ type: 'text', text: 'hello there' }])
})

test('splits text and a closed code block', () => {
  const parts = parseMessageParts('Here you go:\n```python\nprint(1 + 1)\n```\nDone.')
  assert.equal(parts.length, 3)
  assert.deepEqual(parts[0], { type: 'text', text: 'Here you go:\n' })
  assert.deepEqual(parts[1], { type: 'code', language: 'python', code: 'print(1 + 1)', closed: true })
  assert.deepEqual(parts[2], { type: 'text', text: '\nDone.' })
})

test('keeps an unterminated code block usable', () => {
  const parts = parseMessageParts('```js\nconsole.log(1)')
  assert.deepEqual(parts, [{ type: 'code', language: 'js', code: 'console.log(1)', closed: false }])
})

test('handles several code blocks and a missing language', () => {
  const parts = parseMessageParts('```\nfoo\n```\n```Python extra\nbar\n```')
  const code = parts.filter((p) => p.type === 'code')
  assert.equal(code.length, 2)
  assert.equal(code[0].type === 'code' && code[0].language, '')
  assert.equal(code[1].type === 'code' && code[1].language, 'python')
})

test('only python and javascript are runnable', () => {
  assert.equal(runnableLanguage('Python'), 'python')
  assert.equal(runnableLanguage('py'), 'python')
  assert.equal(runnableLanguage('JS'), 'javascript')
  assert.equal(runnableLanguage('rust'), null)
  assert.equal(runnableLanguage(''), null)
})
