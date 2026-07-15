import Layout, { usePuzzleNavigation } from './components/Layout'
import Home from './components/Home'
import Sudoku from './components/Sudoku'

export default function App() {
  const { activePuzzle, navigate } = usePuzzleNavigation()

  return (
    <Layout activePuzzle={activePuzzle} onNavigate={navigate}>
      {activePuzzle === 'home' && <Home />}
      {activePuzzle === 'sudoku' && <Sudoku />}
    </Layout>
  )
}

