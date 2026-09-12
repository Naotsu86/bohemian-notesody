import { createClient } from '@supabase/supabase-js'

const supabaseUrl = 'https://enukaignuqwpjgvkcrlj.supabase.co'
const supabasePublishableKey = 'sb_publishable_dxtUIU-s7t_MJ8oImVUt0w_WXm6FlpJ'

export const supabase = createClient(
  supabaseUrl,
  supabasePublishableKey,
  {
    auth: {
      persistSession: true,
      autoRefreshToken: true,
      detectSessionInUrl: true
    }
  }
)
