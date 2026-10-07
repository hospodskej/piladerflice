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
  get "eshop/:key", to: "home#product", as: :eshop_product

  get "prihlaseni", to: "sessions#new", as: :login
  post "prihlaseni", to: "sessions#create", as: :session
  delete "odhlaseni", to: "sessions#destroy", as: :logout
  get "registrace", to: "registrations#new", as: :new_registration
  post "registrace", to: "registrations#create", as: :registrations
  get "muj-ucet", to: "accounts#show", as: :account
  get "muj-ucet/upravit", to: "accounts#edit", as: :edit_account
  patch "muj-ucet/upravit", to: "accounts#update", as: :update_account
  post "muj-ucet/objednavky/:id/znovu-objednat", to: "accounts#reorder", as: :reorder_order

  resources :cart_items, only: [:create, :update, :destroy]
  get '/kosik', to: 'carts#show', as: :cart
  resources :inquiries, only: [:create]
  get '/obchodni-podminky', to: 'pages#terms', as: :terms
  get '/zasady-cookies', to: 'pages#cookies_policy', as: :cookies_policy

  namespace :admin do
    root to: "dashboard#index"
    resources :orders, only: [:index, :show, :destroy]
    resources :inquiries, only: [:index, :show]
    resources :catalog_products, except: [:show] do
      resources :catalog_variants, only: [:new, :create, :edit, :update, :destroy]
    end
    resources :inquiry_form_options, only: [:index, :create, :update, :destroy] do
      member do
        patch :move_up
        patch :move_down
      end
    end
    resources :promos, except: [:show]
  end

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
