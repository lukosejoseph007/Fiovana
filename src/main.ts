import { invoke } from '@tauri-apps/api/core'
import './styles.css'

let greetInputEl: HTMLInputElement | null
let greetMsgEl: HTMLElement | null

async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Tell TypeScript that the returned value is a string
    const message = (await invoke('greet', {
      name: greetInputEl.value,
    })) as string

    greetMsgEl.textContent = message

    // Tailwind animation
    greetMsgEl.className = 'mt-4 text-lg font-semibold text-primary-600 animate-pulse'

    setTimeout(() => {
      greetMsgEl?.classList.remove('animate-pulse')
    }, 1000)
  }
}

window.addEventListener('DOMContentLoaded', () => {
  greetInputEl = document.querySelector('#greet-input')
  greetMsgEl = document.querySelector('#greet-msg')

  const form = document.querySelector('#greet-form')
  form?.addEventListener('submit', e => {
    e.preventDefault()
    greet()
  })
})
