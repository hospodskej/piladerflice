Rails.application.routes.draw do
  root "home#index"

  get "kontakt", to: "home#kontakt", as: :kontakt
  get "sluzby", to: "home#sluzby", as: :sluzby
  get "sortiment", to: "home#sortiment", as: :sortiment
  get "sortiment/palivove-drevo", to: "home#palivove_drevo", as: :palivove_drevo
  get 'sortiment/stavebni-rezivo', to: 'home#stavebni_rezivo', as: :stavebni_rezivo
  get 'sortiment/truhlarske-rezivo', to: 'home#truhlarske_rezivo', as: :truhlarske_rezivo
  get 'sortiment/okrasne-kamenivo', to: 'home#sortiment_okrasne_kamenivo', as: :okrasne_kamenivo
  get 'sortiment/vyrobni-zbytky', to: 'home#vyrobni_zbytky', as: :vyrobni_zbytky
  get 'eshop', to: "home#eshop", as: :eshop
  get '/eshop/habr', to: 'home#habr', as: :eshop_habr
  get '/eshop/briza', to: 'home#briza', as: :eshop_briza
  get '/eshop/jasan', to: 'home#jasan', as: :eshop_jasan
  get '/eshop/akat', to: 'home#akat', as: :eshop_akat
  get '/eshop/buk', to: 'home#buk', as: :eshop_buk
  get '/eshop/dub', to: 'home#dub', as: :eshop_dub
  get '/eshop/smrk', to: 'home#smrk', as: :eshop_smrk
  get '/eshop/late', to: 'home#late', as: :eshop_late
  get '/eshop/tramy', to: 'home#tramy', as: :eshop_tramy
  get '/eshop/fosny', to: 'home#fosny', as: :eshop_fosny
  get '/eshop/prkna', to: 'home#prkna', as: :eshop_prkna
  get '/eshop/stepka', to: 'home#stepka', as: :eshop_stepka
  get '/eshop/odkory', to: 'home#odkory', as: :eshop_odkory
  get '/eshop/piliny', to: 'home#piliny', as: :eshop_piliny
  get '/eshop/okrasne_kamenivo', to: 'home#eshop_okrasne_kamenivo', as: :eshop_okrasne_kamenivo

  resources :cart_items, only: [:create, :update, :destroy]
  get '/kosik', to: 'carts#show', as: :cart
  get '/obchodni-podminky', to: 'pages#terms', as: :terms

  get '/kosik/doprava', to: 'checkout#shipping', as: :checkout_shipping
  patch '/kosik/doprava', to: 'checkout#update_shipping'
  get '/kosik/udaje', to: 'checkout#details', as: :checkout_details
  patch '/kosik/udaje', to: 'checkout#update_details'
  get '/kosik/souhrn', to: 'checkout#summary', as: :checkout_summary
  post '/kosik/souhrn', to: 'checkout#confirm', as: :checkout_confirm
  get '/kosik/dekujeme', to: 'checkout#confirmation', as: :checkout_confirmation

  # Reveal health status on /up that returns 200 if the app boots with no exceptions, otherwise 500.
  # Can be used by load balancers and uptime monitors to verify that the app is live.
  get "up" => "rails/health#show", as: :rails_health_check


  # Render dynamic PWA files from app/views/pwa/* (remember to link manifest in application.html.erb)
  # get "manifest" => "rails/pwa#manifest", as: :pwa_manifest
  # get "service-worker" => "rails/pwa#service_worker", as: :pwa_service_worker

  # Defines the root path route ("/")
  # root "posts#index"
end
